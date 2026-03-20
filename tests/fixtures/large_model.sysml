// Large SysML v2 model for performance testing
// Auto-generated: 50 module part definitions with metadata, ports, state machines, and actions

package PerfTest {

    import Firmware::*;

    port def DataPort {
        out attribute value : Integer;
    }

    part def Module_001 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_001_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_001_Init { out result : Boolean; }
        action def Module_001_Process { in data : Integer; out result : Integer; }
    }

    part def Module_002 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_002_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_002_Init { out result : Boolean; }
        action def Module_002_Process { in data : Integer; out result : Integer; }
    }

    part def Module_003 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_003_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_003_Init { out result : Boolean; }
        action def Module_003_Process { in data : Integer; out result : Integer; }
    }

    part def Module_004 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_004_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_004_Init { out result : Boolean; }
        action def Module_004_Process { in data : Integer; out result : Integer; }
    }

    part def Module_005 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_005_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_005_Init { out result : Boolean; }
        action def Module_005_Process { in data : Integer; out result : Integer; }
    }

    part def Module_006 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_006_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_006_Init { out result : Boolean; }
        action def Module_006_Process { in data : Integer; out result : Integer; }
    }

    part def Module_007 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_007_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_007_Init { out result : Boolean; }
        action def Module_007_Process { in data : Integer; out result : Integer; }
    }

    part def Module_008 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_008_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_008_Init { out result : Boolean; }
        action def Module_008_Process { in data : Integer; out result : Integer; }
    }

    part def Module_009 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_009_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_009_Init { out result : Boolean; }
        action def Module_009_Process { in data : Integer; out result : Integer; }
    }

    part def Module_010 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_010_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_010_Init { out result : Boolean; }
        action def Module_010_Process { in data : Integer; out result : Integer; }
    }

    part def Module_011 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_011_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_011_Init { out result : Boolean; }
        action def Module_011_Process { in data : Integer; out result : Integer; }
    }

    part def Module_012 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_012_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_012_Init { out result : Boolean; }
        action def Module_012_Process { in data : Integer; out result : Integer; }
    }

    part def Module_013 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_013_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_013_Init { out result : Boolean; }
        action def Module_013_Process { in data : Integer; out result : Integer; }
    }

    part def Module_014 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_014_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_014_Init { out result : Boolean; }
        action def Module_014_Process { in data : Integer; out result : Integer; }
    }

    part def Module_015 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_015_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_015_Init { out result : Boolean; }
        action def Module_015_Process { in data : Integer; out result : Integer; }
    }

    part def Module_016 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_016_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_016_Init { out result : Boolean; }
        action def Module_016_Process { in data : Integer; out result : Integer; }
    }

    part def Module_017 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_017_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_017_Init { out result : Boolean; }
        action def Module_017_Process { in data : Integer; out result : Integer; }
    }

    part def Module_018 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_018_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_018_Init { out result : Boolean; }
        action def Module_018_Process { in data : Integer; out result : Integer; }
    }

    part def Module_019 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_019_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_019_Init { out result : Boolean; }
        action def Module_019_Process { in data : Integer; out result : Integer; }
    }

    part def Module_020 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_020_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_020_Init { out result : Boolean; }
        action def Module_020_Process { in data : Integer; out result : Integer; }
    }

    part def Module_021 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_021_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_021_Init { out result : Boolean; }
        action def Module_021_Process { in data : Integer; out result : Integer; }
    }

    part def Module_022 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_022_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_022_Init { out result : Boolean; }
        action def Module_022_Process { in data : Integer; out result : Integer; }
    }

    part def Module_023 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_023_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_023_Init { out result : Boolean; }
        action def Module_023_Process { in data : Integer; out result : Integer; }
    }

    part def Module_024 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_024_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_024_Init { out result : Boolean; }
        action def Module_024_Process { in data : Integer; out result : Integer; }
    }

    part def Module_025 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_025_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_025_Init { out result : Boolean; }
        action def Module_025_Process { in data : Integer; out result : Integer; }
    }

    part def Module_026 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_026_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_026_Init { out result : Boolean; }
        action def Module_026_Process { in data : Integer; out result : Integer; }
    }

    part def Module_027 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_027_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_027_Init { out result : Boolean; }
        action def Module_027_Process { in data : Integer; out result : Integer; }
    }

    part def Module_028 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_028_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_028_Init { out result : Boolean; }
        action def Module_028_Process { in data : Integer; out result : Integer; }
    }

    part def Module_029 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_029_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_029_Init { out result : Boolean; }
        action def Module_029_Process { in data : Integer; out result : Integer; }
    }

    part def Module_030 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_030_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_030_Init { out result : Boolean; }
        action def Module_030_Process { in data : Integer; out result : Integer; }
    }

    part def Module_031 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_031_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_031_Init { out result : Boolean; }
        action def Module_031_Process { in data : Integer; out result : Integer; }
    }

    part def Module_032 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_032_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_032_Init { out result : Boolean; }
        action def Module_032_Process { in data : Integer; out result : Integer; }
    }

    part def Module_033 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_033_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_033_Init { out result : Boolean; }
        action def Module_033_Process { in data : Integer; out result : Integer; }
    }

    part def Module_034 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_034_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_034_Init { out result : Boolean; }
        action def Module_034_Process { in data : Integer; out result : Integer; }
    }

    part def Module_035 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_035_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_035_Init { out result : Boolean; }
        action def Module_035_Process { in data : Integer; out result : Integer; }
    }

    part def Module_036 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_036_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_036_Init { out result : Boolean; }
        action def Module_036_Process { in data : Integer; out result : Integer; }
    }

    part def Module_037 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_037_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_037_Init { out result : Boolean; }
        action def Module_037_Process { in data : Integer; out result : Integer; }
    }

    part def Module_038 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_038_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_038_Init { out result : Boolean; }
        action def Module_038_Process { in data : Integer; out result : Integer; }
    }

    part def Module_039 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_039_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_039_Init { out result : Boolean; }
        action def Module_039_Process { in data : Integer; out result : Integer; }
    }

    part def Module_040 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_040_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_040_Init { out result : Boolean; }
        action def Module_040_Process { in data : Integer; out result : Integer; }
    }

    part def Module_041 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_041_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_041_Init { out result : Boolean; }
        action def Module_041_Process { in data : Integer; out result : Integer; }
    }

    part def Module_042 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_042_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_042_Init { out result : Boolean; }
        action def Module_042_Process { in data : Integer; out result : Integer; }
    }

    part def Module_043 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_043_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_043_Init { out result : Boolean; }
        action def Module_043_Process { in data : Integer; out result : Integer; }
    }

    part def Module_044 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_044_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_044_Init { out result : Boolean; }
        action def Module_044_Process { in data : Integer; out result : Integer; }
    }

    part def Module_045 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_045_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_045_Init { out result : Boolean; }
        action def Module_045_Process { in data : Integer; out result : Integer; }
    }

    part def Module_046 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_046_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_046_Init { out result : Boolean; }
        action def Module_046_Process { in data : Integer; out result : Integer; }
    }

    part def Module_047 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_047_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_047_Init { out result : Boolean; }
        action def Module_047_Process { in data : Integer; out result : Integer; }
    }

    part def Module_048 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_048_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_048_Init { out result : Boolean; }
        action def Module_048_Process { in data : Integer; out result : Integer; }
    }

    part def Module_049 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_049_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_049_Init { out result : Boolean; }
        action def Module_049_Process { in data : Integer; out result : Integer; }
    }

    part def Module_050 {
        @MemoryModel { allocation = AllocationKind::static_alloc; maxInstances = 1; }
        attribute layer : LayerKind = LayerKind::driver;
        port dataIn : DataPort;
        port dataOut : ~DataPort;

        state def Module_050_SM {
            entry state idle;
            state running;
            state error;
            transition idle_to_running first idle then running;
            transition running_to_error first running then error;
            transition error_to_idle first error then idle;
        }

        action def Module_050_Init { out result : Boolean; }
        action def Module_050_Process { in data : Integer; out result : Integer; }
    }

    // =========================================================================
    // System-level integration: interconnections between modules
    // =========================================================================

    connection def DataConnection {
        end source : DataPort;
        end sink : ~DataPort;
    }

    part def PerfTestSystem {
        // Module instances
        part module_001 : Module_001;
        part module_002 : Module_002;
        part module_003 : Module_003;
        part module_004 : Module_004;
        part module_005 : Module_005;
        part module_006 : Module_006;
        part module_007 : Module_007;
        part module_008 : Module_008;
        part module_009 : Module_009;
        part module_010 : Module_010;
        part module_011 : Module_011;
        part module_012 : Module_012;
        part module_013 : Module_013;
        part module_014 : Module_014;
        part module_015 : Module_015;
        part module_016 : Module_016;
        part module_017 : Module_017;
        part module_018 : Module_018;
        part module_019 : Module_019;
        part module_020 : Module_020;
        part module_021 : Module_021;
        part module_022 : Module_022;
        part module_023 : Module_023;
        part module_024 : Module_024;
        part module_025 : Module_025;
        part module_026 : Module_026;
        part module_027 : Module_027;
        part module_028 : Module_028;
        part module_029 : Module_029;
        part module_030 : Module_030;
        part module_031 : Module_031;
        part module_032 : Module_032;
        part module_033 : Module_033;
        part module_034 : Module_034;
        part module_035 : Module_035;
        part module_036 : Module_036;
        part module_037 : Module_037;
        part module_038 : Module_038;
        part module_039 : Module_039;
        part module_040 : Module_040;
        part module_041 : Module_041;
        part module_042 : Module_042;
        part module_043 : Module_043;
        part module_044 : Module_044;
        part module_045 : Module_045;
        part module_046 : Module_046;
        part module_047 : Module_047;
        part module_048 : Module_048;
        part module_049 : Module_049;
        part module_050 : Module_050;

        // Daisy-chain connections: each module's dataOut connects to the next module's dataIn
        connect module_001.dataOut to module_002.dataIn;
        connect module_002.dataOut to module_003.dataIn;
        connect module_003.dataOut to module_004.dataIn;
        connect module_004.dataOut to module_005.dataIn;
        connect module_005.dataOut to module_006.dataIn;
        connect module_006.dataOut to module_007.dataIn;
        connect module_007.dataOut to module_008.dataIn;
        connect module_008.dataOut to module_009.dataIn;
        connect module_009.dataOut to module_010.dataIn;
        connect module_010.dataOut to module_011.dataIn;
        connect module_011.dataOut to module_012.dataIn;
        connect module_012.dataOut to module_013.dataIn;
        connect module_013.dataOut to module_014.dataIn;
        connect module_014.dataOut to module_015.dataIn;
        connect module_015.dataOut to module_016.dataIn;
        connect module_016.dataOut to module_017.dataIn;
        connect module_017.dataOut to module_018.dataIn;
        connect module_018.dataOut to module_019.dataIn;
        connect module_019.dataOut to module_020.dataIn;
        connect module_020.dataOut to module_021.dataIn;
        connect module_021.dataOut to module_022.dataIn;
        connect module_022.dataOut to module_023.dataIn;
        connect module_023.dataOut to module_024.dataIn;
        connect module_024.dataOut to module_025.dataIn;
        connect module_025.dataOut to module_026.dataIn;
        connect module_026.dataOut to module_027.dataIn;
        connect module_027.dataOut to module_028.dataIn;
        connect module_028.dataOut to module_029.dataIn;
        connect module_029.dataOut to module_030.dataIn;
        connect module_030.dataOut to module_031.dataIn;
        connect module_031.dataOut to module_032.dataIn;
        connect module_032.dataOut to module_033.dataIn;
        connect module_033.dataOut to module_034.dataIn;
        connect module_034.dataOut to module_035.dataIn;
        connect module_035.dataOut to module_036.dataIn;
        connect module_036.dataOut to module_037.dataIn;
        connect module_037.dataOut to module_038.dataIn;
        connect module_038.dataOut to module_039.dataIn;
        connect module_039.dataOut to module_040.dataIn;
        connect module_040.dataOut to module_041.dataIn;
        connect module_041.dataOut to module_042.dataIn;
        connect module_042.dataOut to module_043.dataIn;
        connect module_043.dataOut to module_044.dataIn;
        connect module_044.dataOut to module_045.dataIn;
        connect module_045.dataOut to module_046.dataIn;
        connect module_046.dataOut to module_047.dataIn;
        connect module_047.dataOut to module_048.dataIn;
        connect module_048.dataOut to module_049.dataIn;
        connect module_049.dataOut to module_050.dataIn;
    }
}
