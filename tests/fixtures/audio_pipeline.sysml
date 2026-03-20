// Audio Pipeline module — middleware layer
// Bridges Bluetooth audio reception to I2S output with ring buffer

package Firmware {

    import Firmware::*;

    part def AudioPipeline {
        // ── Metadata annotations ──
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        @ConcurrencyModel { threadSafe = true; isrShared = false; protection = ProtectionKind::mutex; }
        @ErrorHandling { strategy = ErrorKind::result; }
        @ISRSafe { safe = false; }

        attribute layer : LayerKind = LayerKind::middleware;

        // ── Parts (owned sub-components) ──
        part i2s : I2sOutput;
        part bt : BtA2dpSink;
        part led : StatusLed;

        // ── Ports ──
        port audioIn : AudioDataPort;
        port audioWrite : ~I2sWritePort;
        port ledCmd : ~LedCommandPort;

        // ── Internal connections ──
        connect bt.audioOut to audioIn;
        connect audioWrite to i2s.i2sIn;
        connect ledCmd to led.ledIn;

        // ── Flow: PCM data from BT to I2S ──
        flow of Integer
            from bt.audioOut.data
            to i2s.i2sIn.samples;

        // ── Interface actions ──
        action def New {
            in i2s : I2sOutput;
            in bt : BtA2dpSink;
            in led : StatusLed;
            in config : PipelineConfig;
            out pipeline : AudioPipeline;
        }

        action def Run {
            in self : AudioPipeline;
            // Never returns — blocking event loop
        }
    }

    // ── Error type ──
    enum def PipelineError {
        enum I2sError;
        enum BtError;
        enum BufferOverflow;
        enum BufferUnderrun;
    }
}
