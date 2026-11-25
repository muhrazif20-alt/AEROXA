use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::ast::{Program, Statement};
use crate::lexer::tokenize;
use crate::parser::Parser;

#[derive(Debug, Clone)]
pub struct ModuleSource {
    pub name: String,
    pub path: PathBuf,
    pub source: String,
    pub program: Program,
}

#[derive(Debug)]
pub struct ModuleResolver {
    loaded: HashMap<PathBuf, ModuleSource>,
    visiting: HashSet<PathBuf>,
}

impl ModuleResolver {
    pub fn new() -> Self {
        Self {
            loaded: HashMap::new(),
            visiting: HashSet::new(),
        }
    }

    pub fn resolve(
        &mut self,
        main_file: &Path,
        module_root: &Path,
    ) -> Result<Vec<ModuleSource>, String> {
        let main_path = fs::canonicalize(main_file)
            .map_err(|error| {
                format!("Cannot resolve main file: {}", error)
            })?;

        let module_root = fs::canonicalize(module_root)
            .map_err(|error| {
                format!("Cannot resolve module directory: {}", error)
            })?;

        let source = fs::read_to_string(&main_path)
            .map_err(|error| {
                format!("Cannot read main file: {}", error)
            })?;

        let program = self.parse_source(&source)?;

        let mut modules = Vec::new();

        self.resolve_program(
            &program,
            &module_root,
            &mut modules,
        )?;

        Ok(modules)
    }

    fn parse_source(
        &self,
        source: &str,
    ) -> Result<Program, String> {
        let lexer_result = tokenize(source);

        if !lexer_result.diagnostics.is_empty() {
            return Err(format!(
                "Module lexical analysis failed: {:?}",
                lexer_result.diagnostics
            ));
        }

        let (program, diagnostics) =
            Parser::new(lexer_result.tokens).parse();

        if !diagnostics.is_empty() {
            return Err(format!(
                "Module parsing failed: {:?}",
                diagnostics
            ));
        }

        Ok(program)
    }

    fn resolve_program(
        &mut self,
        program: &Program,
        module_root: &Path,
        modules: &mut Vec<ModuleSource>,
    ) -> Result<(), String> {
        for statement in &program.statements {
            if let Statement::Link { module, .. } = statement {
                let module_path = module_root
                    .join(format!("{}.ex", module));

                self.resolve_file(
                    &module_path,
                    module_root,
                    modules,
                )?;
            }
        }

        Ok(())
    }

    fn resolve_file(
        &mut self,
        file_path: &Path,
        module_root: &Path,
        modules: &mut Vec<ModuleSource>,
    ) -> Result<(), String> {
        let absolute_path = fs::canonicalize(file_path)
            .map_err(|error| {
                format!(
                    "Cannot find module '{}': {}",
                    file_path.display(),
                    error
                )
            })?;

        if self.visiting.contains(&absolute_path) {
            return Err(format!(
                "Circular module linking detected: {}",
                absolute_path.display()
            ));
        }

        if self.loaded.contains_key(&absolute_path) {
            return Ok(());
        }

        self.visiting.insert(absolute_path.clone());

        let result = self.load_module(
            &absolute_path,
            module_root,
            modules,
        );

        self.visiting.remove(&absolute_path);

        result
    }

    fn load_module(
        &mut self,
        absolute_path: &Path,
        module_root: &Path,
        modules: &mut Vec<ModuleSource>,
    ) -> Result<(), String> {
        let source = fs::read_to_string(absolute_path)
            .map_err(|error| {
                format!(
                    "Cannot read module '{}': {}",
                    absolute_path.display(),
                    error
                )
            })?;

        let program = self.parse_source(&source)?;

        // Resolve dependencies before loading this module.
        self.resolve_program(
            &program,
            module_root,
            modules,
        )?;

        let name = absolute_path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("unknown")
            .to_string();

        let module = ModuleSource {
            name,
            path: absolute_path.to_path_buf(),
            source,
            program,
        };

        self.loaded.insert(
            absolute_path.to_path_buf(),
            module.clone(),
        );

        // Dependencies are inserted before their parent module.
        modules.push(module);

        Ok(())
    }
}