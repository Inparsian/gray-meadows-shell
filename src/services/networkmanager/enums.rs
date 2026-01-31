#![allow(dead_code)]
/// https://www.networkmanager.dev/docs/api/latest/nm-dbus-types.html#NMState
#[derive(Debug, Clone)]
pub enum NetworkManagerState {
    Unknown = 0,
    Asleep = 10,
    Disconnecting = 20,
    Disconnected = 30,
    Connecting = 40,
    ConnectedLocal = 50,
    ConnectedSite = 60,
    ConnectedGlobal = 70,
}

/// https://www.networkmanager.dev/docs/api/latest/nm-dbus-types.html#NMDeviceState
#[derive(Debug, Clone)]
pub enum NetworkManagerDeviceState {
    Unknown = 0,
    Unmanaged = 10,
    Unavailable = 20,
    Disconnected = 30,
    Prepare = 40,
    Config = 50,
    NeedAuth = 60,
    IPConfig = 70,
    IPCheck = 80,
    Secondaries = 90,
    Activated = 100,
    Deactivating = 110,
    Failed = 120,
}

/// https://www.networkmanager.dev/docs/api/latest/nm-dbus-types.html#NMDeviceStateReason
#[derive(Debug, Clone)]
pub enum NetworkManagerDeviceStateReason {
    None = 0,
    Unknown = 1,
    NowManaged = 2,
    NowUnmanaged = 3,
    ConfigFailed = 4,
    IPConfigUnavailable = 5,
    IPConfigExpired = 6,
    NoSecrets = 7,
    SupplicantDisconnect = 8,
    SupplicantConfigFailed = 9,
    SupplicantFailed = 10,
    SupplicantTimeout = 11,
    PPPStartFailed = 12,
    PPPDisconnect = 13,
    PPPFailed = 14,
    DHCPStartFailed = 15,
    DHCPError = 16,
    DHCPFailed = 17,
    SharedStartFailed = 18,
    SharedFailed = 19,
    AutoIPStartFailed = 20,
    AutoIPError = 21,
    AutoIPFailed = 22,
    ModemBusy = 23,
    ModemNoDialTone = 24,
    ModemNoCarrier = 25,
    ModemDialTimeout = 26,
    ModemDialFailed = 27,
    ModemInitFailed = 28,
    GSMAPNFailed = 29,
    GSMRegistrationNotSearching = 30,
    GSMRegistrationDenied = 31,
    GSMRegistrationTimeout = 32,
    GSMRegistrationFailed = 33,
    GSMPinCheckFailed = 34,
    FirmwareMissing = 35,
    Removed = 36,
    Sleeping = 37,
    ConnectionRemoved = 38,
    UserRequested = 39,
    Carrier = 40,
    ConnectionAssumed = 41,
    SupplicantAvailable = 42,
    ModemNotFound = 43,
    BluetoothFailed = 44,
    GSMSIMNotInserted = 45,
    GSMSIMPinRequired = 46,
    GSMSIMPukRequired = 47,
    GSMSIMWrong = 48,
    InfinibandMode = 49,
    DependencyFailed = 50,
    BR2684Failed = 51,
    ModemManagerUnavailable = 52,
    SSIDNotFound = 53,
    SecondaryConnectionFailed = 54,
    DCBFCoEFailed = 55,
    TeamdControlFailed = 56,
    ModemFailed = 57,
    ModemAvailable = 58,
    SIMPinIncorrect = 59,
    NewActivation = 60,
    ParentChanged = 61,
    ParentManagedChanged = 62,
    OvsdbFailed = 63,
    IpAddressDuplicate = 64,
    IpMethodUnsupported = 65,
    SriovConfigurationFailed = 66,
    PeerNotFound = 67,
    DeviceHandlerFailed = 68,
    UnmanagedByDefault = 69,
    UnmanagedExternalDown = 70,
    UnmanagedLinkNotInitialized = 71,
    UnmanagedQuitting = 72,
    UnmanagedSleeping = 73,
    UnmanagedUserConf = 74,
    UnmanagedUserExplicit = 75,
    UnmanagedUserSettings = 76,
    UnmanagedUserUdev = 77,
}

/// https://www.networkmanager.dev/docs/api/latest/nm-dbus-types.html#NM80211ApFlags
#[derive(Debug, Clone)]
pub enum NetworkManager80211ApFlags {
    None = 0x0000_0000,
    Privacy = 0x0000_0001,
    Wps = 0x0000_0002,
    WpsPbc = 0x0000_0004,
    WpsPin = 0x0000_0008,
}

/// https://www.networkmanager.dev/docs/api/latest/nm-dbus-types.html#NM80211ApSecurityFlags
#[derive(Debug, Clone)]
pub enum NetworkManager80211ApSecurityFlags {
    None = 0x0000_0000,
    PairWEP40 = 0x0000_0001,
    PairWEP104 = 0x0000_0002,
    PairTKIP = 0x0000_0004,
    PairCCMP = 0x0000_0008,
    GroupWEP40 = 0x0000_0010,
    GroupWEP104 = 0x0000_0020,
    GroupTKIP = 0x0000_0040,
    GroupCCMP = 0x0000_0080,
    KeyMgmtPSK = 0x0000_0100,
    KeyMgmt8021X = 0x0000_0200,
    KeyMgmtSAE = 0x0000_0400,
    KeyMgmtOWE = 0x0000_0800,
    KeyMgmtOWETM = 0x0000_1000, // since 1.26
    KeyMgmtEAPSuiteB192 = 0x0000_2000, // since 1.30
}

/// https://www.networkmanager.dev/docs/api/latest/nm-dbus-types.html#NMDeviceInterfaceFlags
#[derive(Debug, Clone)]
pub enum NetworkManagerDeviceInterfaceFlags {
    None = 0,
    Up = 0x1,
    LowerUp = 0x2,
    Promisc = 0x4,
    Carrier = 0x10000,
    LldpCLientEnabled = 0x20000
}