struct ModuleInfo {
    backend: String,
    custom_sections: HashMap<String, Vec<Vec<u8>>>,
    data_initializers: Vec<DataInitializer>,
    elem_initializers: Vec<TableInitializer>,
    em_symbol_map: Option<HashMap<u32, String>>,
    exports: IndexMap<String, ExportIndex>,
    func_assoc: Map<FuncIndex, SigIndex>,
    generate_debug_info: bool,
    globals: Map<LocalGlobalIndex, GlobalInit>,
    imported_functions: Map<ImportedFuncIndex, ImportName>,
    imported_globals: Map<ImportedGlobalIndex, (ImportName, GlobalDescriptor)>,
    imported_memories: Map<ImportedMemoryIndex, (ImportName, MemoryDescriptor)>,
    imported_tables: Map<ImportedTableIndex, (ImportName, TableDescriptor)>,
    memories: Map<LocalMemoryIndex, MemoryDescripto>,
    name_table: StringTable<NameIndex>,
    namespace_table: StringTable<NamespaceIndex>,
    signatures: Map<SigIndex, FuncSig>,
    start_func: Option<FuncIndex>,
    tables: Map<LocalTableIndex, TableDescriptor>,
}