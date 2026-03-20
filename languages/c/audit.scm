;; C audit queries for tree-sitter
;; Extracts functions, structs, and enums

;; Functions
(function_definition
  declarator: (function_declarator
    declarator: (identifier) @fn.name
    parameters: (parameter_list) @fn.params)
  type: (_) @fn.return_type) @fn.def

;; Structs
(struct_specifier
  name: (type_identifier) @struct.name
  body: (field_declaration_list)? @struct.fields) @struct.def

;; Enums
(enum_specifier
  name: (type_identifier) @enum.name
  body: (enumerator_list) @enum.variants) @enum.def
