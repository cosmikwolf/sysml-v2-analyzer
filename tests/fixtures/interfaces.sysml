// Shared port and interface definitions for the firmware system

package Firmware {

    import Firmware::*;

    // ── Port definitions ──

    port def AudioDataPort {
        attribute sampleRate : Integer = 48000;
        attribute bitDepth : Integer = 16;
        attribute channels : Integer = 2;
        out attribute data : Integer[0..*];
    }

    port def StatusPort {
        out attribute 'state' : ConnectionState;
        out attribute discoverable : Boolean;
    }

    port def LedCommandPort {
        in attribute pattern : StatusPattern;
    }

    port def I2sWritePort {
        in attribute samples : Integer[0..*];
        out attribute written : Integer;
    }

    // ── Shared enumerations ──

    enum def ConnectionState {
        enum Disconnected;
        enum Discovering;
        enum Connected;
    }

    enum def StatusPattern {
        enum BtDiscoverable;
        enum BtConnecting;
        enum BtConnected;
        enum AudioUnderrun;
        enum I2sError;
        enum Off;
    }

    // ── Configuration types ──

    attribute def A2dpConfig {
        attribute deviceName : String = "Toothy";
        attribute discoverableTimeoutSecs : Integer = 30;
        attribute preferredSampleRate : Integer = 48000;
    }

    attribute def PipelineConfig {
        attribute bufferSize : Integer = 8192;
        attribute underrunSilence : Boolean = true;
    }
}
