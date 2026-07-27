import Foundation
import Virtualization
import AppKit

// MARK: - Type aliases for C interop

typealias VZVMStateCallbackFunc = @convention(c) (UnsafeMutableRawPointer?, Int32, UnsafeRawPointer?, UnsafeMutableRawPointer?) -> Void

// MARK: - Real VZ Implementation

@available(macOS 12.0, *)
internal class VirtualMachineHost: NSObject, VZVirtualMachineDelegate {
    let vm: VZVirtualMachine
    let queue: DispatchQueue
    var window: NSWindow?
    var machineView: VZVirtualMachineView?
    var ipAddress: String?
    var vmHandle: UnsafeMutableRawPointer?
    var bootMonitorTask: DispatchSourceTimer?

    init(vm: VZVirtualMachine) {
        self.vm = vm
        self.queue = DispatchQueue(label: "com.tinybridge.vm.queue", qos: .userInteractive)
        super.init()
        vm.delegate = self
    }

    func start() {
        queue.async { [weak self] in
            do {
                try self?.vm.start()
                // Start monitoring for network availability
                self?.startBootMonitor()
            } catch {
                NSLog("Failed to start VM: %@", error.localizedDescription)
            }
        }
    }

    func stop() {
        queue.async { [weak self] in
            self?.stopBootMonitor()
            do {
                try self?.vm.stop()
            } catch {
                NSLog("Failed to stop VM: %@", error.localizedDescription)
            }
        }
    }

    func forceStop() {
        queue.async { [weak self] in
            self?.stopBootMonitor()
            try? self?.vm.stop()
        }
    }

    private func startBootMonitor() {
        guard bootMonitorTask == nil else { return }

        let timer = DispatchSource.makeTimerSource(queue: .main)
        timer.schedule(deadline: .now() + 2, repeating: 1.0)
        timer.setEventHandler { [weak self] in
            // Probe for SSH on localhost (port forwarding from VM)
            // For now, just track that VM is running
            if self?.vm.state == .running {
                self?.ipAddress = self?.ipAddress ?? "192.168.105.2"
            }
        }
        timer.resume()
        bootMonitorTask = timer
    }

    private func stopBootMonitor() {
        bootMonitorTask?.cancel()
        bootMonitorTask = nil
    }

    func showWindow() {
        DispatchQueue.main.async { [weak self] in
            guard let self = self else { return }
            if self.window == nil {
                let window = NSWindow(contentRect: NSRect(x: 100, y: 100, width: 1920, height: 1080),
                                     styleMask: [.titled, .closable, .miniaturizable, .resizable],
                                     backing: .buffered, defer: false)
                window.title = "TinyBridge VM"
                let view = VZVirtualMachineView()
                view.virtualMachine = self.vm
                window.contentView = view
                self.window = window
                self.machineView = view
            }
            self.window?.makeKeyAndOrderFront(nil)
        }
    }

    func hideWindow() {
        DispatchQueue.main.async { [weak self] in
            self?.window?.orderOut(nil)
        }
    }

    // MARK: VZVirtualMachineDelegate

    nonisolated func virtualMachine(_ virtualMachine: VZVirtualMachine, didStopWithError error: Error) {
        DispatchQueue.main.async { [weak self] in
            NSLog("VM stopped with error: %@", error.localizedDescription)
            self?.ipAddress = nil
        }
    }

    nonisolated func guestDidStop(_ virtualMachine: VZVirtualMachine) {
        DispatchQueue.main.async { [weak self] in
            NSLog("VM guest stopped")
            self?.ipAddress = nil
        }
    }
}

// MARK: - C FFI Bridge

private var vmInstances: [UnsafeMutableRawPointer: VirtualMachineHost] = [:]
private let vmLock = NSLock()

@_cdecl("tb_version")
public func tb_version() -> UnsafePointer<CChar> {
    let version = "0.1.0"
    return version.withCString { cstr -> UnsafePointer<CChar> in
        let buffer = UnsafeMutablePointer<CChar>.allocate(capacity: version.count + 1)
        strcpy(buffer, cstr)
        return UnsafePointer(buffer)
    }
}

@_cdecl("tb_is_available")
public func tb_is_available() -> Bool {
    #if os(macOS)
    if #available(macOS 12.0, *) {
        return true
    }
    return false
    #else
    return false
    #endif
}

@_cdecl("tb_vm_create")
public func tb_vm_create(_ configPtr: UnsafeRawPointer?) -> UnsafeMutableRawPointer? {
    guard #available(macOS 12.0, *) else { return nil }
    guard let configPtr = configPtr else { return nil }

    let config = configPtr.assumingMemoryBound(to: TBVMConfig.self).pointee

    do {
        var bootConfig = VZLinuxBootLoader()
        if config.kernel_path != nil {
            bootConfig.kernelURL = URL(fileURLWithPath: String(cString: config.kernel_path))
        }
        if config.initrd_path != nil {
            bootConfig.initialRamdiskURL = URL(fileURLWithPath: String(cString: config.initrd_path))
        }
        if config.cmdline != nil {
            bootConfig.commandLine = String(cString: config.cmdline)
        }

        var vmConfig = VZVirtualMachineConfiguration()
        vmConfig.bootLoader = bootConfig
        vmConfig.cpuCount = max(1, Int(config.cpu_count))
        vmConfig.memorySize = config.memory_bytes

        // Storage device
        if config.disk_image_path != nil {
            let diskURL = URL(fileURLWithPath: String(cString: config.disk_image_path))
            let attachment = try VZDiskImageStorageDeviceAttachment(url: diskURL, readOnly: false)
            let storage = VZVirtioBlockDeviceConfiguration(attachment: attachment)
            vmConfig.storageDevices = [storage]
        }

        // Network (NAT)
        let networkDevice = VZVirtioNetworkDeviceConfiguration()
        networkDevice.attachment = VZNATNetworkDeviceAttachment()
        vmConfig.networkDevices = [networkDevice]

        // Graphics device (always enabled, even for headless)
        let scanoutConfig = VZVirtioGraphicsScanoutConfiguration(
            widthInPixels: Int(config.display_width > 0 ? config.display_width : 1920),
            heightInPixels: Int(config.display_height > 0 ? config.display_height : 1080)
        )
        let graphicsConfig = VZVirtioGraphicsDeviceConfiguration()
        graphicsConfig.scanouts = [scanoutConfig]
        vmConfig.graphicsDevices = [graphicsConfig]

        // Input devices
        vmConfig.keyboards = [VZUSBKeyboardConfiguration()]
        vmConfig.pointingDevices = [VZUSBScreenCoordinatePointingDeviceConfiguration()]

        try vmConfig.validate()
        let virtualMachine = try VZVirtualMachine(configuration: vmConfig)

        let host = VirtualMachineHost(vm: virtualMachine)
        let handle = Unmanaged.passRetained(host).toOpaque()
        host.vmHandle = handle

        vmLock.lock()
        vmInstances[handle] = host
        vmLock.unlock()

        return handle
    } catch {
        NSLog("Failed to create VM: %@", error.localizedDescription)
        return nil
    }
}

@_cdecl("tb_vm_start")
public func tb_vm_start(_ vm: UnsafeMutableRawPointer?) -> Int32 {
    guard #available(macOS 12.0, *) else { return -1 }
    guard let vm = vm else { return -1 }

    vmLock.lock()
    let host = vmInstances[vm]
    vmLock.unlock()

    guard let host = host else { return -1 }

    host.start()
    return 0
}

@_cdecl("tb_vm_stop")
public func tb_vm_stop(_ vm: UnsafeMutableRawPointer?) -> Int32 {
    guard #available(macOS 12.0, *) else { return -1 }
    guard let vm = vm else { return -1 }

    vmLock.lock()
    let host = vmInstances[vm]
    vmLock.unlock()

    guard let host = host else { return -1 }

    host.stop()
    return 0
}

@_cdecl("tb_vm_force_stop")
public func tb_vm_force_stop(_ vm: UnsafeMutableRawPointer?) -> Int32 {
    guard #available(macOS 12.0, *) else { return -1 }
    guard let vm = vm else { return -1 }

    vmLock.lock()
    let host = vmInstances[vm]
    vmLock.unlock()

    guard let host = host else { return -1 }

    host.forceStop()
    return 0
}

@_cdecl("tb_vm_destroy")
public func tb_vm_destroy(_ vm: UnsafeMutableRawPointer?) {
    guard let vm = vm else { return }

    vmLock.lock()
    let host = vmInstances.removeValue(forKey: vm)
    vmLock.unlock()

    if let host = host {
        Unmanaged.passRetained(host).release()
    }
}

@_cdecl("tb_vm_add_virtiofs")
public func tb_vm_add_virtiofs(_ vm: UnsafeMutableRawPointer?, _ configPtr: UnsafeRawPointer?) -> Int32 {
    // Note: VirtioFS must be configured at VM creation time; this is a placeholder.
    return 0
}

@_cdecl("tb_vm_get_status")
public func tb_vm_get_status(_ vm: UnsafeMutableRawPointer?, _ outStatusPtr: UnsafeMutableRawPointer?) -> Int32 {
    guard #available(macOS 12.0, *) else { return -1 }
    guard let vm = vm, let outStatusPtr = outStatusPtr else { return -1 }

    vmLock.lock()
    let host = vmInstances[vm]
    vmLock.unlock()

    guard let host = host else { return -1 }

    var status = TBVMStatus()
    // Map VZ state to our state enum
    switch host.vm.state {
    case .stopped:
        status.state = 0  // TB_VM_STATE_STOPPED
    case .running:
        status.state = 2  // TB_VM_STATE_RUNNING
    case .paused:
        status.state = 3  // TB_VM_STATE_STOPPING (treat pause as stopping for now)
    @unknown default:
        status.state = 0
    }

    status.memory_total_bytes = host.vm.configuration.memorySize
    status.memory_used_bytes = 0
    status.cpu_usage_pct = 0

    // Return detected IP or default for NAT
    let ip = host.ipAddress ?? "192.168.105.2"
    let ipCStr = (ip as NSString).utf8String ?? ""
    let len = min(45, strlen(ipCStr))
    let ptr = withUnsafeMutableBytes(of: &status.ip_address) { $0.baseAddress }
    memcpy(ptr, ipCStr, len)

    let statusPtr = outStatusPtr.assumingMemoryBound(to: TBVMStatus.self)
    statusPtr.pointee = status
    return 0
}

@_cdecl("tb_vm_get_ip")
public func tb_vm_get_ip(_ vm: UnsafeMutableRawPointer?, _ buf: UnsafeMutablePointer<CChar>?, _ bufLen: Int) -> Int32 {
    guard #available(macOS 12.0, *) else { return -1 }
    guard let vm = vm, let buf = buf else { return -1 }

    vmLock.lock()
    let host = vmInstances[vm]
    vmLock.unlock()

    guard let host = host, let ip = host.ipAddress else { return -1 }

    let ipCStr = (ip as NSString).utf8String ?? ""
    let len = strlen(ipCStr)
    guard len + 1 <= bufLen else { return -1 }

    memcpy(buf, ipCStr, len)
    buf[len] = 0
    return 0
}

@_cdecl("tb_vm_show_window")
public func tb_vm_show_window(_ vm: UnsafeMutableRawPointer?) -> Int32 {
    guard #available(macOS 12.0, *) else { return -1 }
    guard let vm = vm else { return -1 }

    vmLock.lock()
    let host = vmInstances[vm]
    vmLock.unlock()

    guard let host = host else { return -1 }

    host.showWindow()
    return 0
}

@_cdecl("tb_vm_hide_window")
public func tb_vm_hide_window(_ vm: UnsafeMutableRawPointer?) -> Int32 {
    guard #available(macOS 12.0, *) else { return -1 }
    guard let vm = vm else { return -1 }

    vmLock.lock()
    let host = vmInstances[vm]
    vmLock.unlock()

    guard let host = host else { return -1 }

    host.hideWindow()
    return 0
}
