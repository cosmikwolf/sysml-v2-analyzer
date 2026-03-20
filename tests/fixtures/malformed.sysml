// Intentionally malformed SysML for error recovery testing

package Broken {

    // Missing semicolons
    enum def MissingEnd {
        enum A
        enum B
    }

    // Unclosed brace
    part def UnclosedPart {
        attribute x : Integer;
        port broken : SomeUndefinedPort;

    // Missing closing brace — parser should recover

    // Reference to undefined type
    part def BadRef {
        attribute y : NonExistentType;
        @NonExistentMetadata { value = 42; }
    }

    // Duplicate names
    part def Duplicate {
        attribute z : Integer;
    }
    part def Duplicate {
        attribute z : Boolean;
    }

    // Malformed metadata
    @MemoryModel { allocation = ; }

    // Dangling transition
    state def BrokenFSM {
        state a;
        transition a_to_nowhere
            first a
            accept SomeEvent
            then nonexistent_state;
    }
}
