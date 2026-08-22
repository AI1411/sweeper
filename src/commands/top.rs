use crate::process::list::list_processes;
use crate::style;

pub fn run_top() -> anyhow::Result<()> {
    let mut procs = list_processes();
    println!("{}\n", style::header("CPU"));
    let mut by_cpu = procs.clone();
    by_cpu.sort_by(|a, b| {
        b.cpu
            .partial_cmp(&a.cpu)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for (i, p) in by_cpu.iter().take(10).enumerate() {
        println!(
            "{} {}  {}",
            style::rank(i + 1),
            style::process_name(&p.name),
            style::cpu(p.cpu)
        );
    }
    println!("\n{}\n", style::header("MEMORY"));
    procs.sort_by_key(|p| std::cmp::Reverse(p.memory_bytes));
    for (i, p) in procs.iter().take(10).enumerate() {
        let gb = format!("{:.1} GB", p.memory_bytes as f64 / 1e9);
        println!(
            "{} {}  {}",
            style::rank(i + 1),
            style::process_name(&p.name),
            style::mem(gb)
        );
    }
    Ok(())
}
