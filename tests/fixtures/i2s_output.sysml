// I2S Output module — driver layer
// Drives the Adafruit UDA1334A I2S DAC

package Firmware {

    import Firmware::*;

    part def I2sOutput {
        // ── Metadata annotations ──
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        @ConcurrencyModel { threadSafe = false; isrShared = false; protection = ProtectionKind::none; }
        @ErrorHandling { strategy = ErrorKind::result; }
        @ISRSafe { safe = false; }
        @Ownership {
            owns = ("I2S0 peripheral", "GPIO22 (DOUT)", "GPIO25 (BCK)", "GPIO26 (WS)");
            borrows = ();
        }

        attribute layer : LayerKind = LayerKind::driver;

        // ── Ports ──
        port i2sIn : I2sWritePort;

        // ── Interface actions ──
        action def New {
            in i2s : ScalarValues;  // I2S0 peripheral
            in bck : ScalarValues;  // GPIO25
            in ws : ScalarValues;   // GPIO26
            in dout : ScalarValues; // GPIO22
            out output : I2sOutput;
        }

        action def WriteSamples {
            in self : I2sOutput;
            in data : Integer[0..*];
            out written : Integer;
        }
    }

    // ── Error type ──
    enum def I2sOutputError {
        enum InitFailed;
        enum WriteFailed;
        enum DmaError;
    }
}
