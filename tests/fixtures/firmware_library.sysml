// Firmware Metadata Library
// Defines metadata types, enums, and annotations for firmware-specific concerns.
// This file is the foundation — all module .sysml files import from here.

package Firmware {

    // ── Layer classification ──

    enum def LayerKind {
        enum application;
        enum middleware;
        enum driver;
        enum hal;
        enum pac;
    }

    // ── Memory allocation strategy ──

    enum def AllocationKind {
        enum static_alloc;
        enum heap;
        enum pool;
    }

    // ── Error handling strategy ──

    enum def ErrorKind {
        enum result;
        enum return_code;
        enum exception;
        enum panic;
    }

    // ── Concurrency protection mechanism ──

    enum def ProtectionKind {
        enum none;
        enum mutex;
        enum critical_section;
        enum disable_irq;
        enum atomic;
    }

    // ── Metadata definitions (firmware annotations) ──

    metadata def MemoryModel {
        attribute 'allocation' : AllocationKind;
        attribute maxInstances : Integer;
    }

    metadata def ConcurrencyModel {
        attribute threadSafe : Boolean;
        attribute isrShared : Boolean;
        attribute protection : ProtectionKind;
    }

    metadata def ErrorHandling {
        attribute strategy : ErrorKind;
    }

    metadata def ISRSafe {
        attribute safe : Boolean;
    }

    metadata def Ownership {
        attribute owns : String[0..*];
        attribute borrows : String[0..*];
    }

    // ── Layer dependency rules ──

    metadata def LayerConstraint {
        attribute layer : LayerKind;
        attribute mayDependOn : LayerKind[0..*];
    }

    // ═══════════════════════════════════════════════════════════════
    // UI Metadata — displays, inputs, screens, navigation, indicators
    // ═══════════════════════════════════════════════════════════════

    // ── Display hardware ──

    enum def DisplayKind {
        enum oled;
        enum lcd;
        enum e_ink;
    }

    enum def ColorDepthKind {
        enum mono;
        enum grayscale;
        enum rgb565;
        enum rgb888;
    }

    enum def InterfaceKind {
        enum i2c;
        enum spi;
    }

    metadata def DisplayHardware {
        attribute 'type' : DisplayKind;
        attribute width : Integer;
        attribute height : Integer;
        attribute colorDepth : ColorDepthKind;
        attribute 'interface' : InterfaceKind;
        attribute driver : String;
        attribute orientation : String;
        attribute module : String;
    }

    // ── Input devices ──

    enum def InputKind {
        enum button;
        enum rotary_encoder;
        enum slider;
    }

    enum def ActiveLevel {
        enum low;
        enum high;
    }

    metadata def InputDevice {
        attribute 'type' : InputKind;
        attribute active : ActiveLevel;
        attribute hasButton : Boolean;
        attribute detents : Integer;
        attribute module : String;
    }

    // ── LED hardware ──

    enum def LedType {
        enum single;
        enum bicolor;
        enum rgb;
    }

    metadata def LedHardware {
        attribute 'type' : LedType;
        attribute colors : String[0..*];
        attribute module : String;
    }

    // ── Gesture vocabulary ──

    enum def TriggerKind {
        enum press;
        enum release;
        enum long_press;
        enum rotate_cw;
        enum rotate_ccw;
        enum value_change;
    }

    metadata def GestureTimingDefaults {
        attribute debounce_ms : Integer;
        attribute long_press_ms : Integer;
        attribute double_tap_ms : Integer;
        attribute combo_window_ms : Integer;
        attribute sequence_timeout_ms : Integer;
    }

    metadata def Gesture {
        attribute device : String;
        attribute trigger : TriggerKind;
        attribute window_ms : Integer;
    }

    // ── Assets ──

    enum def AssetSource {
        enum builtin;
        enum file;
        enum inline;
    }

    enum def IconFormat {
        enum mono_1bpp;
        enum grayscale_8bpp;
        enum rgb565;
    }

    metadata def FontAsset {
        attribute family : String;
        attribute size : Integer;
        attribute source : AssetSource;
        attribute file : String;
    }

    metadata def IconAsset {
        attribute width : Integer;
        attribute height : Integer;
        attribute source : AssetSource;
        attribute format : IconFormat;
        attribute file : String;
    }

    // ── Screens and elements ──

    enum def RefreshMode {
        enum 'event';
        enum polling;
    }

    enum def ElementKind {
        enum text;
        enum icon;
        enum progress_bar;
        enum rectangle;
    }

    enum def AlignKind {
        enum left;
        enum center;
        enum right;
    }

    enum def VisibilityOp {
        enum eq;
        enum neq;
        enum gt;
        enum lt;
    }

    metadata def Screen {
        attribute display : String;
        attribute refreshMode : RefreshMode;
        attribute pollInterval_ms : Integer;
    }

    metadata def Element {
        attribute 'type' : ElementKind;
        attribute x : Integer;
        attribute y : Integer;
        attribute width : Integer;
        attribute height : Integer;
        attribute font : String;
        attribute icon : String;
        attribute align : AlignKind;
        attribute scroll : Boolean;
        attribute truncate : Boolean;
        attribute binding_module : String;
        attribute binding_field : String;
        attribute range_min : Integer;
        attribute range_max : Integer;
        attribute visible_module : String;
        attribute visible_field : String;
        attribute visible_op : VisibilityOp;
        attribute visible_value : String;
    }

    // ── Navigation ──

    enum def ActionKind {
        enum navigate;
        enum adjust;
        enum invoke;
        enum menu_next;
        enum menu_prev;
        enum menu_select;
    }

    metadata def Navigation {
        // Marker: tags a state machine as the navigation FSM
    }

    // ── LED indicators ──

    enum def LedPattern {
        enum solid;
        enum blink;
        enum pulse;
    }

    metadata def IndicatorBinding {
        attribute led : String;
        attribute module : String;
        attribute field : String;
    }

    metadata def IndicatorState {
        attribute color : String;
        attribute pattern : LedPattern;
        attribute period_ms : Integer;
        attribute duty_percent : Integer;
    }
}
