import Foundation

// MARK: - Week 3 Stub: C Functions for VZ binding

@_cdecl("tb_version")
public func tb_version() -> UnsafePointer<CChar> {
    let version = "0.1.0"
    let len = version.count + 1
    let buffer = UnsafeMutablePointer<CChar>.allocate(capacity: len)
    _ = version.withCString { src in
        strcpy(buffer, src)
    }
    return UnsafePointer(buffer)
}

@_cdecl("tb_is_available")
public func tb_is_available() -> Bool {
    #if os(macOS)
    return true
    #else
    return false
    #endif
}

@_cdecl("tb_vm_create")
public func tb_vm_create(_ config: UnsafeRawPointer?) -> UnsafeMutableRawPointer? {
    guard config != nil else { return nil }
    let handle = VMStub()
    return Unmanaged.passRetained(handle).toOpaque()
}

@_cdecl("tb_vm_start")
public func tb_vm_start(_ vm: UnsafeMutableRawPointer?) -> Int32 {
    guard let vm = vm else { return -1 }
    let handle = Unmanaged<VMStub>.fromOpaque(vm).takeUnretainedValue()
    handle.start()
    return 0
}

@_cdecl("tb_vm_stop")
public func tb_vm_stop(_ vm: UnsafeMutableRawPointer?) -> Int32 {
    guard let vm = vm else { return -1 }
    let handle = Unmanaged<VMStub>.fromOpaque(vm).takeUnretainedValue()
    handle.stop()
    return 0
}

@_cdecl("tb_vm_force_stop")
public func tb_vm_force_stop(_ vm: UnsafeMutableRawPointer?) -> Int32 {
    guard let vm = vm else { return -1 }
    let handle = Unmanaged<VMStub>.fromOpaque(vm).takeUnretainedValue()
    handle.forceStop()
    return 0
}

@_cdecl("tb_vm_destroy")
public func tb_vm_destroy(_ vm: UnsafeMutableRawPointer?) {
    guard let vm = vm else { return }
    Unmanaged<VMStub>.fromOpaque(vm).release()
}

@_cdecl("tb_vm_add_virtiofs")
public func tb_vm_add_virtiofs(_ vm: UnsafeMutableRawPointer?, _ config: UnsafeRawPointer?) -> Int32 {
    guard vm != nil, config != nil else { return -1 }
    return 0
}

@_cdecl("tb_vm_get_status")
public func tb_vm_get_status(_ vm: UnsafeMutableRawPointer?, _ outStatus: UnsafeMutableRawPointer?) -> Int32 {
    guard let vm = vm, let outStatus = outStatus else { return -1 }
    let handle = Unmanaged<VMStub>.fromOpaque(vm).takeUnretainedValue()
    let status = handle.getStatus()

    let statusPtr = outStatus.assumingMemoryBound(to: TBVMStatusStub.self)
    statusPtr.pointee = status
    return 0
}

@_cdecl("tb_vm_get_ip")
public func tb_vm_get_ip(_ vm: UnsafeMutableRawPointer?, _ buf: UnsafeMutablePointer<CChar>?, _ bufLen: Int) -> Int32 {
    guard let vm = vm, let buf = buf else { return -1 }
    let handle = Unmanaged<VMStub>.fromOpaque(vm).takeUnretainedValue()
    guard let ip = handle.getIPAddress() else { return -1 }

    let ipCString = ip.cString(using: .utf8) ?? []
    guard ipCString.count <= bufLen else { return -1 }

    memcpy(buf, ipCString, ipCString.count - 1)
    buf[ipCString.count - 1] = 0
    return 0
}

// MARK: - Swift implementation stubs

internal struct TBVMStatusStub {
    var state: Int32 = 0
    var cpu_usage_pct: Double = 0
    var memory_used_bytes: UInt64 = 0
    var memory_total_bytes: UInt64 = 0
    var ip_address: (CChar, CChar, CChar, CChar, CChar, CChar, CChar, CChar,
                     CChar, CChar, CChar, CChar, CChar, CChar, CChar, CChar,
                     CChar, CChar, CChar, CChar, CChar, CChar, CChar, CChar,
                     CChar, CChar, CChar, CChar, CChar, CChar, CChar, CChar,
                     CChar, CChar, CChar, CChar, CChar, CChar, CChar, CChar,
                     CChar, CChar, CChar, CChar, CChar, CChar, CChar) = (0, 0, 0, 0, 0, 0, 0, 0,
                                                                             0, 0, 0, 0, 0, 0, 0, 0,
                                                                             0, 0, 0, 0, 0, 0, 0, 0,
                                                                             0, 0, 0, 0, 0, 0, 0, 0,
                                                                             0, 0, 0, 0, 0, 0, 0, 0,
                                                                             0, 0, 0, 0, 0, 0, 0)
}

internal class VMStub {
    var ipAddress: String?
    var stateValue: Int32 = 0  // TB_VM_STATE_STOPPED = 0

    func start() {
        stateValue = 1  // TB_VM_STATE_STARTING
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) { [weak self] in
            self?.stateValue = 2  // TB_VM_STATE_RUNNING
            self?.ipAddress = "192.168.105.2"
        }
    }

    func stop() {
        stateValue = 3  // TB_VM_STATE_STOPPING
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.2) { [weak self] in
            self?.stateValue = 0  // TB_VM_STATE_STOPPED
            self?.ipAddress = nil
        }
    }

    func forceStop() {
        stateValue = 0  // TB_VM_STATE_STOPPED
        ipAddress = nil
    }

    func getStatus() -> TBVMStatusStub {
        var status = TBVMStatusStub()
        status.state = stateValue
        status.memory_total_bytes = 8 * 1024 * 1024 * 1024  // 8GB

        if let ip = ipAddress {
            ip.withCString { cstr in
                let count = min(45, strlen(cstr))
                let ptr = withUnsafeMutableBytes(of: &status.ip_address) { $0.baseAddress }
                memcpy(ptr, cstr, count)
            }
        }

        return status
    }

    func getIPAddress() -> String? {
        ipAddress
    }
}
