// Status LED module — driver layer
// APA102 (DotStar) LED on TinyPICO for status indication

package Firmware {

    import Firmware::*;

    part def StatusLed {
        // ── Metadata annotations ──
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        @ConcurrencyModel { threadSafe = true; isrShared = false; protection = ProtectionKind::mutex; }
        @ErrorHandling { strategy = ErrorKind::result; }
        @ISRSafe { safe = false; }
        @Ownership {
            owns = ("GPIO2 (DotStar data)", "GPIO12 (DotStar clock)");
            borrows = ();
        }

        attribute layer : LayerKind = LayerKind::driver;

        // ── Ports ──
        port ledIn : LedCommandPort;

        // ── LED state machine ──
        state def LedFSM {
            entry; then off;

            state off;
            state solid;
            state blinking;

            transition off_to_solid
                first off
                accept SetSolidEvent
                then solid;

            transition off_to_blinking
                first off
                accept SetPatternEvent
                then blinking;

            transition solid_to_off
                first solid
                accept TurnOffEvent
                then off;

            transition solid_to_blinking
                first solid
                accept SetPatternEvent
                then blinking;

            transition blinking_to_off
                first blinking
                accept TurnOffEvent
                then off;

            transition blinking_to_solid
                first blinking
                accept SetSolidEvent
                then solid;
        }

        // ── Events ──
        attribute def SetSolidEvent {
            attribute r : Integer;
            attribute g : Integer;
            attribute b : Integer;
        }
        attribute def SetPatternEvent {
            attribute pattern : StatusPattern;
        }
        attribute def TurnOffEvent;

        // ── Interface actions ──
        action def New {
            in dataPin : ScalarValues;   // GPIO2
            in clockPin : ScalarValues;  // GPIO12
            out led : StatusLed;
        }

        action def SetPattern {
            in self : StatusLed;
            in pattern : StatusPattern;
        }

        action def SetSolid {
            in self : StatusLed;
            in r : Integer;
            in g : Integer;
            in b : Integer;
        }

        action def Off {
            in self : StatusLed;
        }
    }

    // ── Error type ──
    enum def LedError {
        enum GpioError;
        enum InvalidPattern;
    }
}
