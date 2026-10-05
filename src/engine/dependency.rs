use super::evaluator::Evaluator;
use super::parser::{extract_dependencies, FormulaParser};
use crate::model::cell::{CellCoord, CellError, CellValue};
use crate::model::sheet::Sheet;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Default, Clone)]
pub struct DependencyGraph {
    /// Maps a formula cell -> list of cells it directly reads from
    pub dependencies: HashMap<CellCoord, Vec<CellCoord>>,
    /// Maps a cell -> list of formula cells that read from it
    pub dependents: HashMap<CellCoord, Vec<CellCoord>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(sheet: &Sheet) -> Self {
        let mut graph = Self::new();
        for (coord, cell) in &sheet.cells {
            if let Some(formula) = cell.formula_text() {
                if let Ok(expr) = FormulaParser::parse(formula) {
                    let deps = extract_dependencies(&expr);
                    for dep in &deps {
                        graph
                            .dependents
                            .entry(*dep)
                            .or_default()
                            .push(*coord);
                    }
                    graph.dependencies.insert(*coord, deps);
                }
            }
        }
        graph
    }

    /// Finds all transitive dependents of a cell (cells that need re-evaluation when `root` changes)
    pub fn get_transitive_dependents(&self, root: CellCoord) -> Vec<CellCoord> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(root);

        while let Some(current) = queue.pop_front() {
            if let Some(children) = self.dependents.get(&current) {
                for child in children {
                    if visited.insert(*child) {
                        queue.push_back(*child);
                    }
                }
            }
        }

        // Return in topological order using in-degrees
        self.topological_sort(&visited)
    }

    /// Sorts a subset of cells topologically, detecting cycles
    pub fn topological_sort(&self, subset: &HashSet<CellCoord>) -> Vec<CellCoord> {
        let mut in_degree: HashMap<CellCoord, usize> = HashMap::new();
        for cell in subset {
            let count = self
                .dependencies
                .get(cell)
                .map(|deps| deps.iter().filter(|d| subset.contains(d)).count())
                .unwrap_or(0);
            in_degree.insert(*cell, count);
        }

        let mut queue = VecDeque::new();
        for (cell, &deg) in &in_degree {
            if deg == 0 {
                queue.push_back(*cell);
            }
        }

        let mut sorted = Vec::new();
        while let Some(cell) = queue.pop_front() {
            sorted.push(cell);
            if let Some(children) = self.dependents.get(&cell) {
                for child in children {
                    if subset.contains(child) {
                        if let Some(deg) = in_degree.get_mut(child) {
                            *deg = deg.saturating_sub(1);
                            if *deg == 0 {
                                queue.push_back(*child);
                            }
                        }
                    }
                }
            }
        }

        sorted
    }
}

/// Recalculates all formula cells in a sheet with cycle detection and topological sorting
pub fn recalculate_sheet(sheet: &mut Sheet) {
    let graph = DependencyGraph::build(sheet);
    let all_formula_cells: HashSet<CellCoord> = graph.dependencies.keys().copied().collect();

    // Check for cycles using 3-color DFS
    let mut visited = HashMap::new(); // 0 = unvisited, 1 = in progress (gray), 2 = done (black)
    let mut circular_cells = HashSet::new();

    for &cell in &all_formula_cells {
        if visited.get(&cell).copied().unwrap_or(0) == 0 {
            dfs_cycle_check(&cell, &graph, &mut visited, &mut circular_cells);
        }
    }

    // Set circular errors
    for &cell in &circular_cells {
        if let Some(c) = sheet.get_cell_mut(cell) {
            c.value = CellValue::Error(CellError::Circular);
        }
    }

    // Non-circular cells sorted topologically
    let non_circular: HashSet<CellCoord> = all_formula_cells
        .difference(&circular_cells)
        .copied()
        .collect();

    let eval_order = graph.topological_sort(&non_circular);

    for coord in eval_order {
        if let Some(formula) = sheet.get_cell(coord).and_then(|c| c.formula_text()).map(|s| s.to_string()) {
            let result = Evaluator::evaluate_formula(sheet, coord, &formula);
            if let Some(cell) = sheet.get_cell_mut(coord) {
                cell.value = result;
            }
        }
    }
}

/// Incremental recalculation when a single cell changes
pub fn recalculate_incremental(sheet: &mut Sheet, changed_coord: CellCoord) {
    // If the changed cell itself is a formula, evaluate it first
    if let Some(formula) = sheet.get_cell(changed_coord).and_then(|c| c.formula_text()).map(|s| s.to_string()) {
        let result = Evaluator::evaluate_formula(sheet, changed_coord, &formula);
        if let Some(c) = sheet.get_cell_mut(changed_coord) {
            c.value = result;
        }
    }

    let graph = DependencyGraph::build(sheet);
    let dependents = graph.get_transitive_dependents(changed_coord);

    for coord in dependents {
        if let Some(formula) = sheet.get_cell(coord).and_then(|c| c.formula_text()).map(|s| s.to_string()) {
            let result = Evaluator::evaluate_formula(sheet, coord, &formula);
            if let Some(cell) = sheet.get_cell_mut(coord) {
                cell.value = result;
            }
        }
    }
}

fn dfs_cycle_check(
    curr: &CellCoord,
    graph: &DependencyGraph,
    visited: &mut HashMap<CellCoord, u8>,
    circular: &mut HashSet<CellCoord>,
) {
    visited.insert(*curr, 1); // Gray
    if let Some(deps) = graph.dependencies.get(curr) {
        for dep in deps {
            match visited.get(dep).copied().unwrap_or(0) {
                1 => {
                    // Back-edge found -> cycle!
                    circular.insert(*curr);
                    circular.insert(*dep);
                }
                0 => {
                    dfs_cycle_check(dep, graph, visited, circular);
                    if circular.contains(dep) {
                        circular.insert(*curr);
                    }
                }
                _ => {}
            }
        }
    }
    visited.insert(*curr, 2); // Black
}
