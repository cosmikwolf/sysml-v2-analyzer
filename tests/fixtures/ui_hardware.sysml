// UI subsystem — complete valid model for UI validation testing
// Includes display, inputs, LED, gestures, assets, screens, navigation, indicators

package Firmware {

    import Firmware::*;

    // ════════════════════════════════════════════════════════════════
    // Hardware: display, inputs, LED
    // ════════════════════════════════════════════════════════════════

    part def MainDisplay {
        @DisplayHardware {
            type = DisplayKind::oled;
            width = 128;
            height = 64;
            colorDepth = ColorDepthKind::mono;
            interface = InterfaceKind::i2c;
            driver = "ssd1306";
            orientation = "landscape";
            module = "display";
        }
    }

    part def PlayButton {
        @InputDevice {
            type = InputKind::button;
            active = ActiveLevel::low;
            hasButton = false;
            detents = 0;
            module = "input";
        }
    }

    part def VolumeEncoder {
        @InputDevice {
            type = InputKind::rotary_encoder;
            active = ActiveLevel::low;
            hasButton = true;
            detents = 20;
            module = "input";
        }
    }

    part def StatusLed {
        @LedHardware {
            type = LedType::rgb;
            colors = ("red", "green", "blue");
            module = "status_led";
        }
    }

    // ════════════════════════════════════════════════════════════════
    // Gesture timing defaults
    // ════════════════════════════════════════════════════════════════

    @GestureTimingDefaults {
        debounce_ms = 50;
        long_press_ms = 800;
        double_tap_ms = 300;
        combo_window_ms = 200;
        sequence_timeout_ms = 1000;
    }

    // ════════════════════════════════════════════════════════════════
    // Gesture action definitions
    // ════════════════════════════════════════════════════════════════

    action def BtnPress {
        @Gesture {
            device = "play_button";
            trigger = TriggerKind::press;
            window_ms = 0;
        }
    }

    action def BtnLongPress {
        @Gesture {
            device = "play_button";
            trigger = TriggerKind::long_press;
            window_ms = 800;
        }
    }

    action def EncoderCw {
        @Gesture {
            device = "volume_encoder";
            trigger = TriggerKind::rotate_cw;
            window_ms = 0;
        }
    }

    action def EncoderCcw {
        @Gesture {
            device = "volume_encoder";
            trigger = TriggerKind::rotate_ccw;
            window_ms = 0;
        }
    }

    // ════════════════════════════════════════════════════════════════
    // Assets: fonts, icons
    // ════════════════════════════════════════════════════════════════

    part def Font6x10 {
        @FontAsset {
            family = "fixed";
            size = 10;
            source = AssetSource::builtin;
            file = "";
        }
    }

    part def PlayIcon {
        @IconAsset {
            width = 16;
            height = 16;
            source = AssetSource::file;
            format = IconFormat::mono_1bpp;
            file = "assets/icons/play.raw";
        }
    }

    // ════════════════════════════════════════════════════════════════
    // Screens
    // ════════════════════════════════════════════════════════════════

    part def HomeScreen {
        @Screen {
            display = "main_display";
            refreshMode = RefreshMode::event;
            pollInterval_ms = 0;
        }

        part def title {
            @Element {
                type = ElementKind::text;
                x = 0;
                y = 0;
                width = 128;
                height = 12;
                font = "font_6x10";
                icon = "";
                align = AlignKind::center;
                scroll = false;
                truncate = true;
                binding_module = "bt_a2dp_sink";
                binding_field = "device_name";
                range_min = 0;
                range_max = 0;
                visible_module = "";
                visible_field = "";
                visible_op = VisibilityOp::eq;
                visible_value = "";
            }
        }

        part def status_icon {
            @Element {
                type = ElementKind::icon;
                x = 56;
                y = 20;
                width = 16;
                height = 16;
                font = "";
                icon = "play_icon";
                align = AlignKind::center;
                scroll = false;
                truncate = false;
                binding_module = "";
                binding_field = "";
                range_min = 0;
                range_max = 0;
                visible_module = "";
                visible_field = "";
                visible_op = VisibilityOp::eq;
                visible_value = "";
            }
        }

        part def volume_bar {
            @Element {
                type = ElementKind::progress_bar;
                x = 0;
                y = 52;
                width = 128;
                height = 8;
                font = "";
                icon = "";
                align = AlignKind::left;
                scroll = false;
                truncate = false;
                binding_module = "audio_pipeline";
                binding_field = "volume";
                range_min = 0;
                range_max = 100;
                visible_module = "";
                visible_field = "";
                visible_op = VisibilityOp::eq;
                visible_value = "";
            }
        }
    }

    part def SettingsScreen {
        @Screen {
            display = "main_display";
            refreshMode = RefreshMode::event;
            pollInterval_ms = 0;
        }

        part def heading {
            @Element {
                type = ElementKind::text;
                x = 0;
                y = 0;
                width = 128;
                height = 12;
                font = "font_6x10";
                icon = "";
                align = AlignKind::center;
                scroll = false;
                truncate = false;
                binding_module = "";
                binding_field = "";
                range_min = 0;
                range_max = 0;
                visible_module = "";
                visible_field = "";
                visible_op = VisibilityOp::eq;
                visible_value = "";
            }
        }

        part def brightness_bar {
            @Element {
                type = ElementKind::progress_bar;
                x = 0;
                y = 28;
                width = 128;
                height = 8;
                font = "";
                icon = "";
                align = AlignKind::left;
                scroll = false;
                truncate = false;
                binding_module = "audio_pipeline";
                binding_field = "volume";
                range_min = 0;
                range_max = 100;
                visible_module = "";
                visible_field = "";
                visible_op = VisibilityOp::eq;
                visible_value = "";
            }
        }
    }

    // ════════════════════════════════════════════════════════════════
    // Navigation FSM
    // ════════════════════════════════════════════════════════════════

    state def NavigationFSM {
        @Navigation {}

        entry; then home;

        state home;
        state settings;

        transition home_to_settings
            first home
            accept BtnPress
            then settings;

        transition settings_to_home
            first settings
            accept BtnLongPress
            then home;
    }

    // ════════════════════════════════════════════════════════════════
    // LED indicator FSM
    // ════════════════════════════════════════════════════════════════

    state def StatusIndicatorFSM {
        @IndicatorBinding {
            led = "status_led";
            module = "bt_a2dp_sink";
            field = "connection_state";
        }

        entry; then idle;

        state idle {
            @IndicatorState {
                color = "blue";
                pattern = LedPattern::pulse;
                period_ms = 2000;
                duty_percent = 50;
            }
        }

        state connected {
            @IndicatorState {
                color = "green";
                pattern = LedPattern::solid;
                period_ms = 0;
                duty_percent = 100;
            }
        }

        state error {
            @IndicatorState {
                color = "red";
                pattern = LedPattern::blink;
                period_ms = 500;
                duty_percent = 50;
            }
        }
    }
}
