// UI broken references fixture — should trigger UI003, UI006, UI007, UI008
// Various dangling references to non-existent displays, fonts, icons, devices, LEDs

package Firmware {

    import Firmware::*;

    // ── One valid display (so not ALL refs are broken) ──

    part def RealDisplay {
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

    // ── One valid font (so not ALL refs are broken) ──

    part def Font4x6 {
        @FontAsset {
            family = "fixed";
            size = 6;
            source = AssetSource::builtin;
            file = "";
        }
    }

    // ── UI007: Screen references a display that does not exist ──

    part def OrphanScreen {
        @Screen {
            display = "nonexistent_display";
            refreshMode = RefreshMode::event;
            pollInterval_ms = 0;
        }

        // UI003: Element references a font that does not exist
        part def bad_font_label {
            @Element {
                type = ElementKind::text;
                x = 0;
                y = 0;
                width = 64;
                height = 12;
                font = "missing_font";
                icon = "";
                align = AlignKind::left;
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

        // UI003: Element references an icon that does not exist
        part def bad_icon_elem {
            @Element {
                type = ElementKind::icon;
                x = 0;
                y = 16;
                width = 16;
                height = 16;
                font = "";
                icon = "missing_icon";
                align = AlignKind::left;
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

        // Valid element using the real font (for contrast)
        part def good_label {
            @Element {
                type = ElementKind::text;
                x = 0;
                y = 40;
                width = 64;
                height = 12;
                font = "font_4x6";
                icon = "";
                align = AlignKind::left;
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
    }

    // ── UI006: Gesture references a device that does not exist ──

    action def GhostPress {
        @Gesture {
            device = "ghost_button";
            trigger = TriggerKind::press;
            window_ms = 0;
        }
    }

    // ── UI008: Indicator binding references an LED that does not exist ──

    state def PhantomIndicatorFSM {
        @IndicatorBinding {
            led = "phantom_led";
            module = "bt_a2dp_sink";
            field = "connection_state";
        }

        entry; then idle;

        state idle {
            @IndicatorState {
                color = "red";
                pattern = LedPattern::blink;
                period_ms = 500;
                duty_percent = 50;
            }
        }
    }
}
