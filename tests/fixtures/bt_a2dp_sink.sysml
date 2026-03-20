// Bluetooth A2DP Sink module — driver layer
// Manages Bluetooth audio reception via ESP-IDF A2DP profile

package Firmware {

    import Firmware::*;

    part def BtA2dpSink {
        // ── Metadata annotations ──
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        @ConcurrencyModel { threadSafe = true; isrShared = false; protection = ProtectionKind::mutex; }
        @ErrorHandling { strategy = ErrorKind::result; }
        @ISRSafe { safe = false; }
        @Ownership { owns = ("Bluetooth controller",); borrows = (); }

        attribute layer : LayerKind = LayerKind::driver;

        // ── Ports ──
        port audioOut : ~AudioDataPort;
        port btStatus : StatusPort;

        // ── State machine ──
        state def ConnectionFSM {
            entry; then disconnected;

            state disconnected;
            state discovering;
            state connected;
            state streaming;

            transition disconnected_to_discovering
                first disconnected
                accept StartDiscoveryEvent
                then discovering;

            transition discovering_to_connected
                first discovering
                accept ConnectedEvent
                then connected;

            transition discovering_timeout
                first discovering
                accept TimeoutEvent
                then disconnected;

            transition connected_to_streaming
                first connected
                accept AudioStartEvent
                then streaming;

            transition streaming_to_connected
                first streaming
                accept AudioStopEvent
                then connected;

            transition connected_to_disconnected
                first connected
                accept DisconnectEvent
                then disconnected;

            transition streaming_to_disconnected
                first streaming
                accept DisconnectEvent
                then disconnected;
        }

        // ── Events ──
        attribute def StartDiscoveryEvent;
        attribute def ConnectedEvent;
        attribute def TimeoutEvent;
        attribute def AudioStartEvent;
        attribute def AudioStopEvent;
        attribute def DisconnectEvent;

        // ── Interface actions ──
        action def Init {
            in config : A2dpConfig;
            out result : BtA2dpSink;
        }

        action def Start {
            in self : BtA2dpSink;
        }

        action def Stop {
            in self : BtA2dpSink;
        }

        action def SetDataCallback {
            in self : BtA2dpSink;
            in callback : ScalarValues;
        }

        action def GetConnectionState {
            in self : BtA2dpSink;
            out state : ConnectionState;
        }

        action def IsDiscoverable {
            in self : BtA2dpSink;
            out discoverable : Boolean;
        }
    }

    // ── Error type ──
    enum def A2dpError {
        enum InitFailed;
        enum AlreadyStarted;
        enum NotStarted;
        enum CallbackError;
    }
}
