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
}
