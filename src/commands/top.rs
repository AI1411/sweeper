use crate::process::list::list_processes;

pub fn run_top() -> anyhow::Result<()> {
    let mut procs = list_processes();
    println!("CPU\n");
    let mut by_cpu = procs.clone();
    by_cpu.sort_by(|a, b| {
        b.cpu
            .partial_cmp(&a.cpu)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for (i, p) in by_cpu.iter().take(10).enumerate() {
        println!("{}. {}  {:.0}%", i + 1, p.name, p.cpu);
    }
    println!("\nMEMORY\n");
    procs.sort_by_key(|p| std::cmp::Reverse(p.memory_bytes));
    for (i, p) in procs.iter().take(10).enumerate() {
        println!(
            "{}. {}  {:.1} GB",
            i + 1,
            p.name,
            p.memory_bytes as f64 / 1e9
        );
    }
    Ok(())
}
