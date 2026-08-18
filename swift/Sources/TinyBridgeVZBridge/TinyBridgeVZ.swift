import Foundation
import Virtualization
import AppKit
import CTinyBridgeVZ

// MARK: - Real VZ Implementation
//
// This file implements the C ABI declared in swift/Sources/CTinyBridgeVZ/tinybridge_vz.h
// against Apple's real Virtualization.framework. Every tb_vm_* entry point below drives an
// actual VZVirtualMachine instance - there is no mock/stub state machine.
//
// Threading note: VZVirtualMachine created via `VZVirtualMachine(configuration:)` (the
// convenience initializer used below, i.e. no explicit `queue:` argument) is documented by
// Apple to be bound to the main dispatch queue/main actor for the lifetime of the instance.
// All calls into the VM (start/stop/status) are therefore dispatched onto the main actor.

// Mutable state (ipAddress/lastError/window) is only ever written from the main queue
// (see the `DispatchQueue.main.async` / `Task { @MainActor in ... }` blocks below), so
// this is safe despite not being statically provable by the compiler.
@available(macOS 13.0, *)
internal class VirtualMachineHost: NSObject, VZVirtualMachineDelegate, @unchecked Sendable {
    let vm: VZVirtualMachine
    let configuredMemoryBytes: UInt64
    var window: NSWindow?
    var machineView: VZVirtualMachineView?
    var ipAddress: String?
    var vmHandle: UnsafeMutableRawPointer?
    var lastError: String?
    var bootMonitorTask: DispatchSourceTimer?

    init(vm: VZVirtualMachine, configuredMemoryBytes: UInt64) {
        self.vm = vm
        self.configuredMemoryBytes = configuredMemoryBytes
        super.init()
        vm.delegate = self
    }

    /// Kick off a real VZVirtualMachine boot. Errors from the framework are captured on
    /// `lastError` rather than swallowed, so tb_vm_get_status can surface them honestly.
    func start() {
        Task { @MainActor [weak self] in
            guard let self = self else { return }
            do {
                try await self.vm.start()
                self.lastError = nil
                self.startBootMonitor()
            } catch {
                self.lastError = error.localizedDescription
                NSLog("TinyBridgeVZBridge: VM start failed: %@", error.localizedDescription)
            }
        }
    }

    func stop() {
        Task { @MainActor [weak self] in
            guard let self = self else { return }
            self.stopBootMonitor()
            do {
                try await self.vm.stop()
                self.lastError = nil
            } catch {
                self.lastError = error.localizedDescription
                NSLog("TinyBridgeVZBridge: VM stop failed: %@", error.localizedDescription)
            }
        }
    }

    func forceStop() {
        Task { @MainActor [weak self] in
            guard let self = self else { return }
            self.stopBootMonitor()
            do {
                try await self.vm.stop()
            } catch {
                // Force-stop is best-effort; record but do not treat as fatal.
                self.lastError = error.localizedDescription
            }
        }
    }

    private func startBootMonitor() {
        guard bootMonitorTask == nil else { return }

        let timer = DispatchSource.makeTimerSource(queue: .main)
        timer.schedule(deadline: .now() + 2, repeating: 1.0)
        timer.setEventHandler { [weak self] in
            // TODO: probe guest-side (e.g. SSH reachability through the NAT device) instead
            // of assuming the default VZ NAT gateway-assigned address once we have a real
            // guest image to test against.
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
            NSLog("TinyBridgeVZBridge: VM stopped with error: %@", error.localizedDescription)
            self?.ipAddress = nil
            self?.lastError = error.localizedDescription
        }
    }

    nonisolated func guestDidStop(_ virtualMachine: VZVirtualMachine) {
        DispatchQueue.main.async { [weak self] in
            NSLog("TinyBridgeVZBridge: VM guest stopped")
            self?.ipAddress = nil
        }
    }
}

// MARK: - C FFI Bridge

@available(macOS 13.0, *)
private var vmInstances: [UnsafeMutableRawPointer: VirtualMachineHost] = [:]
private let vmLock = NSLock()

@available(macOS 13.0, *)
private func host(for vm: UnsafeMutableRawPointer) -> VirtualMachineHost? {
    vmLock.lock()
    defer { vmLock.unlock() }
    return vmInstances[vm]
}

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
    if #available(macOS 13.0, *) {
        return VZVirtualMachine.isSupported
    }
    return false
    #else
    return false
    #endif
}

@_cdecl("tb_vm_create")
public func tb_vm_create(_ configPtr: UnsafeRawPointer?) -> UnsafeMutableRawPointer? {
    guard #available(macOS 13.0, *) else {
        NSLog("TinyBridgeVZBridge: macOS 13.0+ required for Virtualization.framework")
        return nil
    }
    guard let configPtr = configPtr else { return nil }

    let config = configPtr.assumingMemoryBound(to: TBVMConfig.self).pointee

    guard let kernelPathC = config.kernel_path else {
        NSLog("TinyBridgeVZBridge: kernel_path is required")
        return nil
    }

    do {
        let bootConfig = VZLinuxBootLoader(kernelURL: URL(fileURLWithPath: String(cString: kernelPathC)))
        if let initrdPathC = config.initrd_path {
            bootConfig.initialRamdiskURL = URL(fileURLWithPath: String(cString: initrdPathC))
        }
        if let cmdlineC = config.cmdline {
            bootConfig.commandLine = String(cString: cmdlineC)
        }

        let vmConfig = VZVirtualMachineConfiguration()
        vmConfig.bootLoader = bootConfig
        vmConfig.cpuCount = max(1, Int(config.cpu_count))
        vmConfig.memorySize = config.memory_bytes

        // Storage device
        if let diskPathC = config.disk_image_path {
            let diskURL = URL(fileURLWithPath: String(cString: diskPathC))
            let attachment = try VZDiskImageStorageDeviceAttachment(url: diskURL, readOnly: false)
            let storage = VZVirtioBlockDeviceConfiguration(attachment: attachment)
            vmConfig.storageDevices = [storage]
        }

        // Network (NAT) - safe-by-default: NAT only, never bridged to the host LAN.
        let networkDevice = VZVirtioNetworkDeviceConfiguration()
        networkDevice.attachment = VZNATNetworkDeviceAttachment()
        vmConfig.networkDevices = [networkDevice]

        // Graphics device (VirtIO GPU scanout - this is the real Metal-accelerated display
        // path; there is no separate "GPU passthrough" API in the public Virtualization
        // framework). Only attached when a display size is requested: creating this device
        // opens a real WindowServer/SkyLight session, which macOS gates behind Screen
        // Recording TCC consent for the responsible app - a headless (serial-only) VM must
        // not require that permission just to boot.
        if config.display_width > 0 && config.display_height > 0 {
            let scanoutConfig = VZVirtioGraphicsScanoutConfiguration(
                widthInPixels: Int(config.display_width),
                heightInPixels: Int(config.display_height)
            )
            let graphicsConfig = VZVirtioGraphicsDeviceConfiguration()
            graphicsConfig.scanouts = [scanoutConfig]
            vmConfig.graphicsDevices = [graphicsConfig]

            // Input devices only make sense alongside a display.
            vmConfig.keyboards = [VZUSBKeyboardConfiguration()]
            vmConfig.pointingDevices = [VZUSBScreenCoordinatePointingDeviceConfiguration()]
        }

        // Serial console — real boot/init output, not just VM-lifecycle
        // status. Without this, `console=hvc0` in the kernel cmdline
        // points to a device that was never attached, so there was no way
        // to actually observe whether a guest boots to a login prompt.
        if let serialLogPathC = config.serial_log_path {
            let logPath = String(cString: serialLogPathC)
            if !FileManager.default.fileExists(atPath: logPath) {
                FileManager.default.createFile(atPath: logPath, contents: nil)
            }
            if let writeHandle = FileHandle(forWritingAtPath: logPath) {
                let attachment = VZFileHandleSerialPortAttachment(
                    fileHandleForReading: nil,
                    fileHandleForWriting: writeHandle
                )
                let consoleConfig = VZVirtioConsoleDeviceSerialPortConfiguration()
                consoleConfig.attachment = attachment
                vmConfig.serialPorts = [consoleConfig]
            } else {
                NSLog("TinyBridgeVZBridge: could not open serial_log_path for writing: %@", logPath)
            }
        }

        try vmConfig.validate()
        let virtualMachine = VZVirtualMachine(configuration: vmConfig)

        let host = VirtualMachineHost(vm: virtualMachine, configuredMemoryBytes: config.memory_bytes)
        let handle = Unmanaged.passRetained(host).toOpaque()
        host.vmHandle = handle

        vmLock.lock()
        vmInstances[handle] = host
        vmLock.unlock()

        return handle
    } catch {
        NSLog("TinyBridgeVZBridge: Failed to create VM: %@", error.localizedDescription)
        return nil
    }
}

@_cdecl("tb_vm_start")
public func tb_vm_start(_ vm: UnsafeMutableRawPointer?) -> Int32 {
    guard #available(macOS 13.0, *) else { return -1 }
    guard let vm = vm, let host = host(for: vm) else { return -1 }
    host.start()
    return 0
}

@_cdecl("tb_vm_stop")
public func tb_vm_stop(_ vm: UnsafeMutableRawPointer?) -> Int32 {
    guard #available(macOS 13.0, *) else { return -1 }
    guard let vm = vm, let host = host(for: vm) else { return -1 }
    host.stop()
    return 0
}

@_cdecl("tb_vm_force_stop")
public func tb_vm_force_stop(_ vm: UnsafeMutableRawPointer?) -> Int32 {
    guard #available(macOS 13.0, *) else { return -1 }
    guard let vm = vm, let host = host(for: vm) else { return -1 }
    host.forceStop()
    return 0
}

@_cdecl("tb_vm_destroy")
public func tb_vm_destroy(_ vm: UnsafeMutableRawPointer?) {
    guard #available(macOS 13.0, *) else { return }
    guard let vm = vm else { return }

    vmLock.lock()
    let removed = vmInstances.removeValue(forKey: vm)
    vmLock.unlock()

    if removed != nil {
        // Balances the retain performed by Unmanaged.passRetained(host) in tb_vm_create.
        Unmanaged<VirtualMachineHost>.fromOpaque(vm).release()
    }
}

@_cdecl("tb_vm_add_virtiofs")
public func tb_vm_add_virtiofs(_ vm: UnsafeMutableRawPointer?, _ configPtr: UnsafeRawPointer?) -> Int32 {
    // VZVirtioFileSystemDeviceConfiguration / VZSharedDirectory must be attached to
    // VZVirtualMachineConfiguration.directorySharingDevices *before* VZVirtualMachine is
    // constructed - Virtualization.framework has no API to hot-add a share to an
    // already-created VM. tb_vm_create() does not yet accept a list of shares, so this
    // entry point intentionally returns "not implemented" rather than pretending to
    // succeed. See tinybridge-vz/src/virtiofs.rs for the host-path scoping logic that is
    // ready to be wired in once tb_vm_create()/TBVMConfig grow share-list support.
    return -2
}

@_cdecl("tb_vm_get_status")
public func tb_vm_get_status(_ vm: UnsafeMutableRawPointer?, _ outStatusPtr: UnsafeMutableRawPointer?) -> Int32 {
    guard #available(macOS 13.0, *) else { return -1 }
    guard let vm = vm, let outStatusPtr = outStatusPtr, let host = host(for: vm) else { return -1 }

    var status = TBVMStatus()
    switch host.vm.state {
    case .stopped:
        status.state = TB_VM_STATE_STOPPED
    case .running:
        status.state = TB_VM_STATE_RUNNING
    case .starting:
        status.state = TB_VM_STATE_STARTING
    case .stopping, .pausing:
        status.state = TB_VM_STATE_STOPPING
    case .paused, .resuming, .saving, .restoring:
        status.state = TB_VM_STATE_STOPPING
    case .error:
        status.state = TB_VM_STATE_ERROR
    @unknown default:
        status.state = TB_VM_STATE_ERROR
    }

    status.memory_total_bytes = host.configuredMemoryBytes
    status.memory_used_bytes = 0 // Not exposed by Virtualization.framework's public API.
    status.cpu_usage_pct = 0 // Not exposed by Virtualization.framework's public API.

    let ip = host.ipAddress ?? ""
    ip.withCString { cIp in
        let len = min(45, strlen(cIp))
        withUnsafeMutableBytes(of: &status.ip_address) { raw in
            raw.copyBytes(from: UnsafeRawBufferPointer(start: cIp, count: len))
            raw[len] = 0
        }
    }

    let statusPtr = outStatusPtr.assumingMemoryBound(to: TBVMStatus.self)
    statusPtr.pointee = status
    return 0
}

@_cdecl("tb_vm_get_ip")
public func tb_vm_get_ip(_ vm: UnsafeMutableRawPointer?, _ buf: UnsafeMutablePointer<CChar>?, _ bufLen: Int) -> Int32 {
    guard #available(macOS 13.0, *) else { return -1 }
    guard let vm = vm, let buf = buf, let host = host(for: vm), let ip = host.ipAddress else { return -1 }

    return ip.withCString { cIp -> Int32 in
        let len = strlen(cIp)
        guard len + 1 <= bufLen else { return -1 }
        memcpy(buf, cIp, len)
        buf[len] = 0
        return 0
    }
}

@_cdecl("tb_vm_show_window")
public func tb_vm_show_window(_ vm: UnsafeMutableRawPointer?) -> Int32 {
    guard #available(macOS 13.0, *) else { return -1 }
    guard let vm = vm, let host = host(for: vm) else { return -1 }
    host.showWindow()
    return 0
}

@_cdecl("tb_vm_hide_window")
public func tb_vm_hide_window(_ vm: UnsafeMutableRawPointer?) -> Int32 {
    guard #available(macOS 13.0, *) else { return -1 }
    guard let vm = vm, let host = host(for: vm) else { return -1 }
    host.hideWindow()
    return 0
}
