#[derive(Debug)]
pub struct ProcStat {
    pub cpu_name: String,
    pub cpu_sum: f32,
    pub cpu_idle: f32,
}

impl Default for ProcStat {
    fn default() -> ProcStat {
        ProcStat {
            cpu_name: "cpu".to_string(),
            cpu_sum: 0.0,
            cpu_idle: 0.0,
        }
    }
}

/// Parse the /proc/stat file that contains the usage
pub fn parse_proc_file(proc: String) -> Vec<ProcStat> {
    let lines: Vec<_> = proc.lines().collect();
    let mut procs: Vec<ProcStat> = Vec::<ProcStat>::new();
    for l in lines {
        if l.starts_with("cpu") {
            let mut columns: Vec<_> = l.split(' ').collect();

            // Remove first index if cpu starts with "cpu  " because the two spaces count as a
            // column
            if l.starts_with("cpu  ") {
                columns.remove(0);
            }

            let mut proc_struct: ProcStat = ProcStat {
                cpu_name: columns[0].to_string(),
                // fill in the rest of the values
                ..Default::default()
            };

            for col in &columns {
                let parse = col.parse::<f32>();
                if let Ok(num) = parse {
                    proc_struct.cpu_sum += num;
                }
            }

            let num = columns[4]
                .parse::<f32>()
                .expect("Should have parsed float from /proc/stat file.");
            proc_struct.cpu_idle = num;
            procs.push(proc_struct);
        } else {
            // Leave after lines are not prefixed with cpu
            break;
        }
    }
    procs
}
