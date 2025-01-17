use std::fmt::Display;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use hap::HapType;

#[derive(Copy, Clone, Hash, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum HapTypeWrapper {
    Unknown,
    AccessCodeControlPoint,
    AccessCodeSupportedConfiguration,
    AccessControlLevel,
    AccessoryFlags,
    AccessoryIdentifier,
    Active,
    ActiveIdentifier,
    ActivityInterval,
    AdministratorOnlyAccess,
    AirParticulateDensity,
    AirParticulateSize,
    AirplayEnable,
    ApplicationMatchingIdentifier,
    AssetUpdateReadiness,
    AudioFeedback,
    BatteryLevel,
    Brightness,
    ButtonEvent,
    CcaEnergyDetectThreshold,
    CcaSignalDetectThreshold,
    CameraOperatingModeIndicator,
    CarbonDioxideDetected,
    CarbonDioxideLevel,
    CarbonDioxidePeakLevel,
    CarbonMonoxideDetected,
    CarbonMonoxideLevel,
    CarbonMonoxidePeakLevel,
    CharacteristicValueActiveTransitionCount,
    CharacteristicValueTransitionControl,
    ChargingState,
    ClosedCaptions,
    CloudRelayControlPoint,
    CloudRelayCurrentState,
    CloudRelayEnableStatus,
    ColorTemperature,
    ConfigurationState,
    ConfiguredName,
    ContactSensorState,
    CoolingThresholdTemperature,
    CurrentAirPurifierState,
    CurrentAirQuality,
    CurrentDoorState,
    CurrentFanState,
    CurrentHeaterCoolerState,
    CurrentHeatingCoolingState,
    CurrentHorizontalTiltAngle,
    CurrentHumidifierDehumidifierState,
    CurrentLightLevel,
    CurrentMediaState,
    CurrentPosition,
    CurrentRelativeHumidity,
    CurrentSlatState,
    CurrentTemperature,
    CurrentTiltAngle,
    CurrentTransport,
    CurrentVerticalTiltAngle,
    CurrentVisibilityState,
    CurrentWaterLevel,
    DataStreamHapTransport,
    DataStreamHapTransportInterrupt,
    DigitalZoom,
    DisplayOrder,
    EventRetransmissionMaximum,
    EventSnapshotsActive,
    EventTransmissionCounters,
    FilterChangeIndication,
    FilterLifeLevel,
    FilterResetChangeIndication,
    FirmwareRevision,
    FirmwareUpdateReadiness,
    FirmwareUpdateStatus,
    HardwareFinish,
    HardwareRevision,
    HeartBeat,
    HeatingThresholdTemperature,
    HoldPosition,
    HomekitCameraActive,
    Hue,
    Identifier,
    Identify,
    ImageMirroring,
    ImageRotation,
    InUse,
    InputDeviceType,
    InputSourceType,
    IsConfigured,
    LabelIndex,
    LabelNamespace,
    LeakDetected,
    ListPairings,
    LockControlPoint,
    LockCurrentState,
    LockLastKnownAction,
    LockManagementAutoSecurityTimeout,
    LockPhysicalControls,
    LockTargetState,
    Logs,
    MacRetransmissionMaximum,
    MacTransmissionCounters,
    ManagedNetworkEnable,
    ManuallyDisabled,
    Manufacturer,
    MaximumTransmitPower,
    Model,
    MotionDetected,
    MultifunctionButton,
    Mute,
    NfcAccessControlPoint,
    NfcAccessSupportedConfiguration,
    Name,
    NetworkAccessViolationControl,
    NetworkClientControl,
    NetworkClientStatusControl,
    NightVision,
    NitrogenDioxideDensity,
    ObstructionDetected,
    OccupancyDetected,
    OperatingStateResponse,
    OpticalZoom,
    OutletInUse,
    OzoneDensity,
    Pm10Density,
    Pm2_5Density,
    PairSetup,
    PairVerify,
    PairingFeatures,
    PasswordSetting,
    PeriodicSnapshotsActive,
    PictureMode,
    Ping,
    PositionState,
    PowerModeSelection,
    PowerState,
    ProductData,
    ProgramMode,
    ProgrammableSwitchEvent,
    ProgrammableSwitchOutputState,
    ReceivedSignalStrengthIndication,
    ReceiverSensitivity,
    RelativeHumidityDehumidifierThreshold,
    RelativeHumidityHumidifierThreshold,
    RemainingDuration,
    RemoteKey,
    RotationDirection,
    RotationSpeed,
    RouterStatus,
    Saturation,
    SecuritySystemAlarmType,
    SecuritySystemCurrentState,
    SecuritySystemTargetState,
    SelectedAudioStreamConfiguration,
    SelectedCameraRecordingConfiguration,
    SelectedDiagnosticsModes,
    SelectedStreamConfiguration,
    SerialNumber,
    ServiceSignature,
    SetDuration,
    SetupDataStreamTransport,
    SetupEndpoint,
    SetupTransferTransport,
    SignalToNoiseRatio,
    SiriEnable,
    SiriEndpointSessionStatus,
    SiriEngineVersion,
    SiriInputType,
    SiriLightOnUse,
    SiriListening,
    SiriTouchToUse,
    SlatType,
    SleepDiscoveryMode,
    SleepInterval,
    SmokeDetected,
    SoftwareRevision,
    StagedFirmwareVersion,
    StatusActive,
    StatusFault,
    StatusJammed,
    StatusLowBattery,
    StatusTampered,
    StreamingStatus,
    SulphurDioxideDensity,
    SupportedAssetTypes,
    SupportedAudioRecordingConfiguration,
    SupportedAudioStreamConfiguration,
    SupportedCameraRecordingConfiguration,
    SupportedCharacteristicValueTransitionConfiguration,
    SupportedDataStreamTransportConfiguration,
    SupportedDiagnosticsModes,
    SupportedDiagnosticsSnapshot,
    SupportedFirmwareUpdateConfiguration,
    SupportedRtpConfiguration,
    SupportedRouterConfiguration,
    SupportedTargetConfiguration,
    SupportedTransferTransportConfiguration,
    SupportedVideoRecordingConfiguration,
    SupportedVideoStreamConfiguration,
    SwingMode,
    TargetAirPurifierState,
    TargetDoorState,
    TargetFanState,
    TargetHeaterCoolerState,
    TargetHeatingCoolingState,
    TargetHorizontalTiltAngle,
    TargetHumidifierDehumidifierState,
    TargetListConfiguration,
    TargetMediaState,
    TargetPosition,
    TargetRelativeHumidity,
    TargetTemperature,
    TargetTiltAngle,
    TargetVerticalTiltAngle,
    TargetVisibilityState,
    TemperatureDisplayUnits,
    ThirdPartyCameraActive,
    ThreadControlPoint,
    ThreadNodeCapabilities,
    ThreadOpenthreadVersion,
    ThreadStatus,
    TransmitPower,
    ValveType,
    Version,
    VideoAnalysisActive,
    VolatileOrganicCompoundDensity,
    Volume,
    VolumeControlType,
    VolumeSelector,
    WanConfigurationList,
    WanStatusList,
    WakeConfiguration,
    WiFiCapabilities,
    WiFiConfigurationControl,
    WiFiSatelliteStatus,
    RecordingAudioActive,
    AccessCode,
    AccessControl,
    AccessoryInformation,
    AccessoryMetrics,
    AccessoryRuntimeInformation,
    AirPurifier,
    AirQualitySensor,
    AssetUpdate,
    Assistant,
    AudioStreamManagement,
    Battery,
    CameraOperatingMode,
    CameraRecordingManagement,
    CameraStreamManagement,
    CarbonDioxideSensor,
    CarbonMonoxideSensor,
    CloudRelay,
    ContactSensor,
    DataStreamTransportManagement,
    Diagnostics,
    Door,
    Doorbell,
    Fan,
    FanV2,
    Faucet,
    FilterMaintenance,
    GarageDoorOpener,
    HeaterCooler,
    HumidifierDehumidifier,
    HumiditySensor,
    InputSource,
    IrrigationSystem,
    Label,
    LeakSensor,
    LightSensor,
    Lightbulb,
    LockManagement,
    LockMechanism,
    Microphone,
    MotionSensor,
    NfcAccessService,
    OccupancySensor,
    Outlet,
    Pairing,
    PowerManagement,
    ProtocolInformation,
    SecuritySystem,
    Siri,
    SiriEndpoint,
    Slats,
    SmartSpeaker,
    SmokeSensor,
    Speaker,
    StatefulProgrammableSwitch,
    StatelessProgrammableSwitch,
    Switch,
    TargetControl,
    TargetControlManagement,
    Television,
    TemperatureSensor,
    Thermostat,
    ThreadTransport,
    TransferTransportManagement,
    Valve,
    WiFiRouter,
    WiFiSatellite,
    WiFiTransport,
    Window,
    WindowCovering,
}

impl From<HapType> for HapTypeWrapper {
    fn from(value: HapType) -> Self {
        let str = format!("{:?}", value);
        str.parse().unwrap()
    }
}

impl Display for HapTypeWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for HapTypeWrapper {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let str = format!("\"{}\"", value);
        let val: Self = serde_json::from_str(&str)?;
        Ok(val)
    }
}


impl HapTypeWrapper {
    fn to_sort_uuid(&self) -> String {
        match self {
            HapTypeWrapper::Unknown => "unknown".into(),
            // HapTypeWrapper::Custom(uuid) => uuid.to_hyphenated().to_string(),
            HapTypeWrapper::AccessCodeControlPoint => "262".into(),
            HapTypeWrapper::AccessCodeSupportedConfiguration => "261".into(),
            HapTypeWrapper::AccessControlLevel => "E5".into(),
            HapTypeWrapper::AccessoryFlags => "A6".into(),
            HapTypeWrapper::AccessoryIdentifier => "57".into(),
            HapTypeWrapper::Active => "B0".into(),
            HapTypeWrapper::ActiveIdentifier => "E7".into(),
            HapTypeWrapper::ActivityInterval => "23B".into(),
            HapTypeWrapper::AdministratorOnlyAccess => "1".into(),
            HapTypeWrapper::AirParticulateDensity => "64".into(),
            HapTypeWrapper::AirParticulateSize => "65".into(),
            HapTypeWrapper::AirplayEnable => "25B".into(),
            HapTypeWrapper::ApplicationMatchingIdentifier => "A4".into(),
            HapTypeWrapper::AssetUpdateReadiness => "269".into(),
            HapTypeWrapper::AudioFeedback => "5".into(),
            HapTypeWrapper::BatteryLevel => "68".into(),
            HapTypeWrapper::Brightness => "8".into(),
            HapTypeWrapper::ButtonEvent => "126".into(),
            HapTypeWrapper::CcaEnergyDetectThreshold => "246".into(),
            HapTypeWrapper::CcaSignalDetectThreshold => "245".into(),
            HapTypeWrapper::CameraOperatingModeIndicator => "21D".into(),
            HapTypeWrapper::CarbonDioxideDetected => "92".into(),
            HapTypeWrapper::CarbonDioxideLevel => "93".into(),
            HapTypeWrapper::CarbonDioxidePeakLevel => "94".into(),
            HapTypeWrapper::CarbonMonoxideDetected => "69".into(),
            HapTypeWrapper::CarbonMonoxideLevel => "90".into(),
            HapTypeWrapper::CarbonMonoxidePeakLevel => "91".into(),
            HapTypeWrapper::CharacteristicValueActiveTransitionCount => "24B".into(),
            HapTypeWrapper::CharacteristicValueTransitionControl => "143".into(),
            HapTypeWrapper::ChargingState => "8F".into(),
            HapTypeWrapper::ClosedCaptions => "DD".into(),
            HapTypeWrapper::CloudRelayControlPoint => "5E".into(),
            HapTypeWrapper::CloudRelayCurrentState => "5C".into(),
            HapTypeWrapper::CloudRelayEnableStatus => "5B".into(),
            HapTypeWrapper::ColorTemperature => "CE".into(),
            HapTypeWrapper::ConfigurationState => "263".into(),
            HapTypeWrapper::ConfiguredName => "E3".into(),
            HapTypeWrapper::ContactSensorState => "6A".into(),
            HapTypeWrapper::CoolingThresholdTemperature => "D".into(),
            HapTypeWrapper::CurrentAirPurifierState => "A9".into(),
            HapTypeWrapper::CurrentAirQuality => "95".into(),
            HapTypeWrapper::CurrentDoorState => "E".into(),
            HapTypeWrapper::CurrentFanState => "AF".into(),
            HapTypeWrapper::CurrentHeaterCoolerState => "B1".into(),
            HapTypeWrapper::CurrentHeatingCoolingState => "F".into(),
            HapTypeWrapper::CurrentHorizontalTiltAngle => "6C".into(),
            HapTypeWrapper::CurrentHumidifierDehumidifierState => "B3".into(),
            HapTypeWrapper::CurrentLightLevel => "6B".into(),
            HapTypeWrapper::CurrentMediaState => "E0".into(),
            HapTypeWrapper::CurrentPosition => "6D".into(),
            HapTypeWrapper::CurrentRelativeHumidity => "10".into(),
            HapTypeWrapper::CurrentSlatState => "AA".into(),
            HapTypeWrapper::CurrentTemperature => "11".into(),
            HapTypeWrapper::CurrentTiltAngle => "C1".into(),
            HapTypeWrapper::CurrentTransport => "22B".into(),
            HapTypeWrapper::CurrentVerticalTiltAngle => "6E".into(),
            HapTypeWrapper::CurrentVisibilityState => "135".into(),
            HapTypeWrapper::CurrentWaterLevel => "B5".into(),
            HapTypeWrapper::DataStreamHapTransport => "138".into(),
            HapTypeWrapper::DataStreamHapTransportInterrupt => "139".into(),
            HapTypeWrapper::DigitalZoom => "11D".into(),
            HapTypeWrapper::DisplayOrder => "136".into(),
            HapTypeWrapper::EventRetransmissionMaximum => "23D".into(),
            HapTypeWrapper::EventSnapshotsActive => "223".into(),
            HapTypeWrapper::EventTransmissionCounters => "23E".into(),
            HapTypeWrapper::FilterChangeIndication => "AC".into(),
            HapTypeWrapper::FilterLifeLevel => "AB".into(),
            HapTypeWrapper::FilterResetChangeIndication => "AD".into(),
            HapTypeWrapper::FirmwareRevision => "52".into(),
            HapTypeWrapper::FirmwareUpdateReadiness => "234".into(),
            HapTypeWrapper::FirmwareUpdateStatus => "235".into(),
            HapTypeWrapper::HardwareFinish => "26C".into(),
            HapTypeWrapper::HardwareRevision => "53".into(),
            HapTypeWrapper::HeartBeat => "24A".into(),
            HapTypeWrapper::HeatingThresholdTemperature => "12".into(),
            HapTypeWrapper::HoldPosition => "6F".into(),
            HapTypeWrapper::HomekitCameraActive => "21B".into(),
            HapTypeWrapper::Hue => "13".into(),
            HapTypeWrapper::Identifier => "E6".into(),
            HapTypeWrapper::Identify => "14".into(),
            HapTypeWrapper::ImageMirroring => "11F".into(),
            HapTypeWrapper::ImageRotation => "11E".into(),
            HapTypeWrapper::InUse => "D2".into(),
            HapTypeWrapper::InputDeviceType => "DC".into(),
            HapTypeWrapper::InputSourceType => "DB".into(),
            HapTypeWrapper::IsConfigured => "D6".into(),
            HapTypeWrapper::LabelIndex => "CB".into(),
            HapTypeWrapper::LabelNamespace => "CD".into(),
            HapTypeWrapper::LeakDetected => "70".into(),
            HapTypeWrapper::ListPairings => "50".into(),
            HapTypeWrapper::LockControlPoint => "19".into(),
            HapTypeWrapper::LockCurrentState => "1D".into(),
            HapTypeWrapper::LockLastKnownAction => "1C".into(),
            HapTypeWrapper::LockManagementAutoSecurityTimeout => "1A".into(),
            HapTypeWrapper::LockPhysicalControls => "A7".into(),
            HapTypeWrapper::LockTargetState => "1E".into(),
            HapTypeWrapper::Logs => "1F".into(),
            HapTypeWrapper::MacRetransmissionMaximum => "247".into(),
            HapTypeWrapper::MacTransmissionCounters => "248".into(),
            HapTypeWrapper::ManagedNetworkEnable => "215".into(),
            HapTypeWrapper::ManuallyDisabled => "227".into(),
            HapTypeWrapper::Manufacturer => "20".into(),
            HapTypeWrapper::MaximumTransmitPower => "243".into(),
            HapTypeWrapper::Model => "21".into(),
            HapTypeWrapper::MotionDetected => "22".into(),
            HapTypeWrapper::MultifunctionButton => "26B".into(),
            HapTypeWrapper::Mute => "11A".into(),
            HapTypeWrapper::NfcAccessControlPoint => "264".into(),
            HapTypeWrapper::NfcAccessSupportedConfiguration => "265".into(),
            HapTypeWrapper::Name => "23".into(),
            HapTypeWrapper::NetworkAccessViolationControl => "21F".into(),
            HapTypeWrapper::NetworkClientControl => "20C".into(),
            HapTypeWrapper::NetworkClientStatusControl => "20D".into(),
            HapTypeWrapper::NightVision => "11B".into(),
            HapTypeWrapper::NitrogenDioxideDensity => "C4".into(),
            HapTypeWrapper::ObstructionDetected => "24".into(),
            HapTypeWrapper::OccupancyDetected => "71".into(),
            HapTypeWrapper::OperatingStateResponse => "232".into(),
            HapTypeWrapper::OpticalZoom => "11C".into(),
            HapTypeWrapper::OutletInUse => "26".into(),
            HapTypeWrapper::OzoneDensity => "C3".into(),
            HapTypeWrapper::Pm10Density => "C7".into(),
            HapTypeWrapper::Pm2_5Density => "C6".into(),
            HapTypeWrapper::PairSetup => "4C".into(),
            HapTypeWrapper::PairVerify => "4E".into(),
            HapTypeWrapper::PairingFeatures => "4F".into(),
            HapTypeWrapper::PasswordSetting => "E4".into(),
            HapTypeWrapper::PeriodicSnapshotsActive => "225".into(),
            HapTypeWrapper::PictureMode => "E2".into(),
            HapTypeWrapper::Ping => "23C".into(),
            HapTypeWrapper::PositionState => "72".into(),
            HapTypeWrapper::PowerModeSelection => "DF".into(),
            HapTypeWrapper::PowerState => "25".into(),
            HapTypeWrapper::ProductData => "220".into(),
            HapTypeWrapper::ProgramMode => "D1".into(),
            HapTypeWrapper::ProgrammableSwitchEvent => "73".into(),
            HapTypeWrapper::ProgrammableSwitchOutputState => "74".into(),
            HapTypeWrapper::ReceivedSignalStrengthIndication => "23F".into(),
            HapTypeWrapper::ReceiverSensitivity => "244".into(),
            HapTypeWrapper::RelativeHumidityDehumidifierThreshold => "C9".into(),
            HapTypeWrapper::RelativeHumidityHumidifierThreshold => "CA".into(),
            HapTypeWrapper::RemainingDuration => "D4".into(),
            HapTypeWrapper::RemoteKey => "E1".into(),
            HapTypeWrapper::RotationDirection => "28".into(),
            HapTypeWrapper::RotationSpeed => "29".into(),
            HapTypeWrapper::RouterStatus => "20E".into(),
            HapTypeWrapper::Saturation => "2F".into(),
            HapTypeWrapper::SecuritySystemAlarmType => "8E".into(),
            HapTypeWrapper::SecuritySystemCurrentState => "66".into(),
            HapTypeWrapper::SecuritySystemTargetState => "67".into(),
            HapTypeWrapper::SelectedAudioStreamConfiguration => "128".into(),
            HapTypeWrapper::SelectedCameraRecordingConfiguration => "209".into(),
            HapTypeWrapper::SelectedDiagnosticsModes => "24D".into(),
            HapTypeWrapper::SelectedStreamConfiguration => "117".into(),
            HapTypeWrapper::SerialNumber => "30".into(),
            HapTypeWrapper::ServiceSignature => "A5".into(),
            HapTypeWrapper::SetDuration => "D3".into(),
            HapTypeWrapper::SetupDataStreamTransport => "131".into(),
            HapTypeWrapper::SetupEndpoint => "118".into(),
            HapTypeWrapper::SetupTransferTransport => "201".into(),
            HapTypeWrapper::SignalToNoiseRatio => "241".into(),
            HapTypeWrapper::SiriEnable => "255".into(),
            HapTypeWrapper::SiriEndpointSessionStatus => "254".into(),
            HapTypeWrapper::SiriEngineVersion => "25A".into(),
            HapTypeWrapper::SiriInputType => "132".into(),
            HapTypeWrapper::SiriLightOnUse => "258".into(),
            HapTypeWrapper::SiriListening => "256".into(),
            HapTypeWrapper::SiriTouchToUse => "257".into(),
            HapTypeWrapper::SlatType => "C0".into(),
            HapTypeWrapper::SleepDiscoveryMode => "E8".into(),
            HapTypeWrapper::SleepInterval => "23A".into(),
            HapTypeWrapper::SmokeDetected => "76".into(),
            HapTypeWrapper::SoftwareRevision => "54".into(),
            HapTypeWrapper::StagedFirmwareVersion => "249".into(),
            HapTypeWrapper::StatusActive => "75".into(),
            HapTypeWrapper::StatusFault => "77".into(),
            HapTypeWrapper::StatusJammed => "78".into(),
            HapTypeWrapper::StatusLowBattery => "79".into(),
            HapTypeWrapper::StatusTampered => "7A".into(),
            HapTypeWrapper::StreamingStatus => "120".into(),
            HapTypeWrapper::SulphurDioxideDensity => "C5".into(),
            HapTypeWrapper::SupportedAssetTypes => "268".into(),
            HapTypeWrapper::SupportedAudioRecordingConfiguration => "207".into(),
            HapTypeWrapper::SupportedAudioStreamConfiguration => "115".into(),
            HapTypeWrapper::SupportedCameraRecordingConfiguration => "205".into(),
            HapTypeWrapper::SupportedCharacteristicValueTransitionConfiguration => "144".into(),
            HapTypeWrapper::SupportedDataStreamTransportConfiguration => "130".into(),
            HapTypeWrapper::SupportedDiagnosticsModes => "24C".into(),
            HapTypeWrapper::SupportedDiagnosticsSnapshot => "238".into(),
            HapTypeWrapper::SupportedFirmwareUpdateConfiguration => "233".into(),
            HapTypeWrapper::SupportedRtpConfiguration => "116".into(),
            HapTypeWrapper::SupportedRouterConfiguration => "210".into(),
            HapTypeWrapper::SupportedTargetConfiguration => "123".into(),
            HapTypeWrapper::SupportedTransferTransportConfiguration => "202".into(),
            HapTypeWrapper::SupportedVideoRecordingConfiguration => "206".into(),
            HapTypeWrapper::SupportedVideoStreamConfiguration => "114".into(),
            HapTypeWrapper::SwingMode => "B6".into(),
            HapTypeWrapper::TargetAirPurifierState => "A8".into(),
            HapTypeWrapper::TargetDoorState => "32".into(),
            HapTypeWrapper::TargetFanState => "BF".into(),
            HapTypeWrapper::TargetHeaterCoolerState => "B2".into(),
            HapTypeWrapper::TargetHeatingCoolingState => "33".into(),
            HapTypeWrapper::TargetHorizontalTiltAngle => "7B".into(),
            HapTypeWrapper::TargetHumidifierDehumidifierState => "B4".into(),
            HapTypeWrapper::TargetListConfiguration => "124".into(),
            HapTypeWrapper::TargetMediaState => "137".into(),
            HapTypeWrapper::TargetPosition => "7C".into(),
            HapTypeWrapper::TargetRelativeHumidity => "34".into(),
            HapTypeWrapper::TargetTemperature => "35".into(),
            HapTypeWrapper::TargetTiltAngle => "C2".into(),
            HapTypeWrapper::TargetVerticalTiltAngle => "7D".into(),
            HapTypeWrapper::TargetVisibilityState => "134".into(),
            HapTypeWrapper::TemperatureDisplayUnits => "36".into(),
            HapTypeWrapper::ThirdPartyCameraActive => "21C".into(),
            HapTypeWrapper::ThreadControlPoint => "704".into(),
            HapTypeWrapper::ThreadNodeCapabilities => "702".into(),
            HapTypeWrapper::ThreadOpenthreadVersion => "706".into(),
            HapTypeWrapper::ThreadStatus => "703".into(),
            HapTypeWrapper::TransmitPower => "242".into(),
            HapTypeWrapper::ValveType => "D5".into(),
            HapTypeWrapper::Version => "37".into(),
            HapTypeWrapper::VideoAnalysisActive => "229".into(),
            HapTypeWrapper::VolatileOrganicCompoundDensity => "C8".into(),
            HapTypeWrapper::Volume => "119".into(),
            HapTypeWrapper::VolumeControlType => "E9".into(),
            HapTypeWrapper::VolumeSelector => "EA".into(),
            HapTypeWrapper::WanConfigurationList => "211".into(),
            HapTypeWrapper::WanStatusList => "212".into(),
            HapTypeWrapper::WakeConfiguration => "222".into(),
            HapTypeWrapper::WiFiCapabilities => "22C".into(),
            HapTypeWrapper::WiFiConfigurationControl => "22D".into(),
            HapTypeWrapper::WiFiSatelliteStatus => "21E".into(),
            HapTypeWrapper::RecordingAudioActive => "226".into(),
            HapTypeWrapper::AccessCode => "260".into(),
            HapTypeWrapper::AccessControl => "DA".into(),
            HapTypeWrapper::AccessoryInformation => "3E".into(),
            HapTypeWrapper::AccessoryMetrics => "270".into(),
            HapTypeWrapper::AccessoryRuntimeInformation => "239".into(),
            HapTypeWrapper::AirPurifier => "BB".into(),
            HapTypeWrapper::AirQualitySensor => "8D".into(),
            HapTypeWrapper::AssetUpdate => "267".into(),
            HapTypeWrapper::Assistant => "26A".into(),
            HapTypeWrapper::AudioStreamManagement => "127".into(),
            HapTypeWrapper::Battery => "96".into(),
            HapTypeWrapper::CameraOperatingMode => "21A".into(),
            HapTypeWrapper::CameraRecordingManagement => "204".into(),
            HapTypeWrapper::CameraStreamManagement => "110".into(),
            HapTypeWrapper::CarbonDioxideSensor => "97".into(),
            HapTypeWrapper::CarbonMonoxideSensor => "7F".into(),
            HapTypeWrapper::CloudRelay => "5A".into(),
            HapTypeWrapper::ContactSensor => "80".into(),
            HapTypeWrapper::DataStreamTransportManagement => "129".into(),
            HapTypeWrapper::Diagnostics => "237".into(),
            HapTypeWrapper::Door => "81".into(),
            HapTypeWrapper::Doorbell => "121".into(),
            HapTypeWrapper::Fan => "40".into(),
            HapTypeWrapper::FanV2 => "B7".into(),
            HapTypeWrapper::Faucet => "D7".into(),
            HapTypeWrapper::FilterMaintenance => "BA".into(),
            HapTypeWrapper::GarageDoorOpener => "41".into(),
            HapTypeWrapper::HeaterCooler => "BC".into(),
            HapTypeWrapper::HumidifierDehumidifier => "BD".into(),
            HapTypeWrapper::HumiditySensor => "82".into(),
            HapTypeWrapper::InputSource => "D9".into(),
            HapTypeWrapper::IrrigationSystem => "CF".into(),
            HapTypeWrapper::Label => "CC".into(),
            HapTypeWrapper::LeakSensor => "83".into(),
            HapTypeWrapper::LightSensor => "84".into(),
            HapTypeWrapper::Lightbulb => "43".into(),
            HapTypeWrapper::LockManagement => "44".into(),
            HapTypeWrapper::LockMechanism => "45".into(),
            HapTypeWrapper::Microphone => "112".into(),
            HapTypeWrapper::MotionSensor => "85".into(),
            HapTypeWrapper::NfcAccessService => "266".into(),
            HapTypeWrapper::OccupancySensor => "86".into(),
            HapTypeWrapper::Outlet => "47".into(),
            HapTypeWrapper::Pairing => "55".into(),
            HapTypeWrapper::PowerManagement => "221".into(),
            HapTypeWrapper::ProtocolInformation => "A2".into(),
            HapTypeWrapper::SecuritySystem => "7E".into(),
            HapTypeWrapper::Siri => "133".into(),
            HapTypeWrapper::SiriEndpoint => "253".into(),
            HapTypeWrapper::Slats => "B9".into(),
            HapTypeWrapper::SmartSpeaker => "228".into(),
            HapTypeWrapper::SmokeSensor => "87".into(),
            HapTypeWrapper::Speaker => "113".into(),
            HapTypeWrapper::StatefulProgrammableSwitch => "88".into(),
            HapTypeWrapper::StatelessProgrammableSwitch => "89".into(),
            HapTypeWrapper::Switch => "49".into(),
            HapTypeWrapper::TargetControl => "125".into(),
            HapTypeWrapper::TargetControlManagement => "122".into(),
            HapTypeWrapper::Television => "D8".into(),
            HapTypeWrapper::TemperatureSensor => "8A".into(),
            HapTypeWrapper::Thermostat => "4A".into(),
            HapTypeWrapper::ThreadTransport => "701".into(),
            HapTypeWrapper::TransferTransportManagement => "203".into(),
            HapTypeWrapper::Valve => "D0".into(),
            HapTypeWrapper::WiFiRouter => "20A".into(),
            HapTypeWrapper::WiFiSatellite => "20F".into(),
            HapTypeWrapper::WiFiTransport => "22A".into(),
            HapTypeWrapper::Window => "8B".into(),
            HapTypeWrapper::WindowCovering => "8C".into(),
        }
    }
}

impl Into<HapType> for HapTypeWrapper {
    fn into(self) -> HapType {
        HapType::from_str(self.to_sort_uuid().as_str()).unwrap()
    }
}

#[cfg(test)]
pub mod test {
    use serde_json::json;
    use hap::HapType;
    use crate::hap_type_wrapper::HapTypeWrapper;

    #[test]
    fn test() {
        let c = HapTypeWrapper::SecuritySystemTargetState;
        let a = json!(c);
        let b: HapType = c.into();
        println!("{:?}", a.as_str());
    }
}