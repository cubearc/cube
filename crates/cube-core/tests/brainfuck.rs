use cube_core::{
    errors::ErrorCode,
    esolang::brainfuck::{
        self as bf, ComparisonLimits, Diagnostics, EofPolicy, Limits, Options, Program, StopReason,
    },
    operations,
};
fn report(source: &str, bits: u8, steps: u64) -> bf::Report {
    Program::parse(source)
        .unwrap()
        .report(
            &[],
            Options {
                cell_bits: bits,
                ..Options::default()
            },
            Limits {
                max_instructions: steps,
                ..Limits::default()
            },
            Diagnostics {
                tape_len: 3,
                ..Diagnostics::default()
            },
        )
        .unwrap()
}
#[test]
fn commands_binary_io_loops_and_exact_dispatch() {
    assert_eq!(
        bf::execute("++++++++[>++++++++<-]>+.", &[], Limits::default()).unwrap(),
        b"A"
    );
    assert_eq!(
        bf::execute(
            " ",
            &[],
            Limits {
                max_instructions: 0,
                ..Limits::default()
            }
        )
        .unwrap(),
        b""
    );
    for (source, count, cells, pointer) in [
        ("[+]", 1, [0, 0, 0], 0),
        ("++[>++[>+<-]<-]", 37, [0, 0, 4], 0),
        ("+[]", 10, [1, 0, 0], 0),
        ("+[+]", 512, [0, 0, 0], 0),
        ("+>++<-", 6, [0, 2, 0], 0),
    ] {
        let r = report(source, 8, count);
        assert_eq!(r.instructions, count, "{source}");
        assert_eq!(r.final_pointer, pointer);
        assert_eq!(r.tape.iter().map(|c| c.value).collect::<Vec<_>>(), cells);
        if source == "+[]" {
            assert!(!r.completed);
            assert_eq!(r.stop_reason, StopReason::InstructionLimit);
        } else {
            assert!(r.completed, "{source}: {:?}", r.stop_reason);
        }
    }
    let input: Vec<u8> = (0..=255).collect();
    assert_eq!(
        bf::execute(&",.".repeat(256), &input, Limits::default()).unwrap(),
        input
    );
    for bits in [8, 16, 32] {
        let r = report("-.", bits, 2);
        assert!(r.completed);
        assert_eq!(r.output, [255]);
        assert_eq!(r.tape[0].value, u32::MAX >> (32 - bits));
        let r = Program::parse("-,.")
            .unwrap()
            .report(
                &[128],
                Options {
                    cell_bits: bits,
                    eof: EofPolicy::Error,
                },
                Limits::default(),
                Diagnostics {
                    tape_len: 1,
                    ..Diagnostics::default()
                },
            )
            .unwrap();
        assert_eq!(r.tape[0].value, 128);
        assert_eq!(r.output, [128]);
        assert_eq!(r.input_consumed, 1);
        assert_eq!(report("-+", bits, 2).tape[0].value, 0);
    }
    assert_eq!(report(&"+".repeat(256), 8, 256).tape[0].value, 0);
    assert_eq!(report(&"+".repeat(256), 16, 256).tape[0].value, 256);
    assert_eq!(report(&"+".repeat(256), 32, 256).tape[0].value, 256);
}
#[test]
fn syntax_is_strict_and_all_brackets_validate_before_execution() {
    for (source, offset) in [
        ("[", 0),
        ("[]]", 2),
        ("][", 0),
        ("[[]", 0),
        ("[x]", 1),
        ("[雪]", 1),
        ("\u{b}", 0),
        ("\u{a0}", 0),
    ] {
        let e = Program::parse(source).unwrap_err();
        assert_eq!(e.code, ErrorCode::InvalidInput);
        assert_eq!(e.offset, Some(offset));
    }
    assert!(Program::parse(" \t\r\n+-<>.,[]").is_ok());
    assert!(Program::parse(&" ".repeat(bf::MAX_SOURCE_BYTES)).is_ok());
    assert_eq!(
        Program::parse(&" ".repeat(bf::MAX_SOURCE_BYTES + 1))
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert!(Program::parse(&"+".repeat(bf::MAX_COMMANDS)).is_ok());
    assert_eq!(
        Program::parse(&"+".repeat(bf::MAX_COMMANDS + 1))
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert!(Program::parse(&format!("{}{}", "[".repeat(256), "]".repeat(256))).is_ok());
    assert_eq!(
        Program::parse(&"[".repeat(257)).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
}
#[test]
fn pointer_eof_and_budget_failures_are_contextual_and_never_byte_successes() {
    let e = bf::execute("<", &[], Limits::default()).unwrap_err();
    assert_eq!(e.parameter, Some("pointer"));
    assert_eq!(e.offset, Some(0));
    let r = report(&format!("{}+", ">".repeat(29999)), 8, 30000);
    assert!(r.completed);
    assert_eq!(r.final_pointer, 29999);
    let r = report(&">".repeat(30000), 8, 30000);
    assert!(!r.completed);
    assert_eq!(r.stop_reason, StopReason::PointerOutOfBounds);
    assert_eq!(r.instructions, 30000);
    let p = Program::parse("-,.").unwrap();
    assert_eq!(
        p.execute(&[], Options::default(), Limits::default())
            .unwrap_err()
            .parameter,
        Some("input")
    );
    for (policy, expected) in [(EofPolicy::Zero, 0), (EofPolicy::Unchanged, 255)] {
        assert_eq!(
            p.execute(
                &[],
                Options {
                    eof: policy,
                    ..Options::default()
                },
                Limits::default()
            )
            .unwrap(),
            [expected]
        );
    }
    for value in ["error", "zero", "unchanged"] {
        assert!(EofPolicy::parse(value).is_ok());
    }
    assert!(EofPolicy::parse("implicit").is_err());
    let p = Program::parse(".+[]").unwrap();
    let r = p
        .report(
            &[],
            Options::default(),
            Limits {
                max_instructions: 7,
                ..Limits::default()
            },
            Diagnostics::default(),
        )
        .unwrap();
    assert!(!r.completed);
    assert_eq!(r.output, [0]);
    assert_eq!(r.instructions, 7);
    assert!(
        p.execute(
            &[],
            Options::default(),
            Limits {
                max_instructions: 7,
                ..Limits::default()
            }
        )
        .is_err()
    );
    assert!(report("+", 8, 1).completed);
    assert_eq!(report("+", 8, 0).instructions, 0);
    let p = Program::parse("..").unwrap();
    let r = p
        .report(
            &[],
            Options::default(),
            Limits {
                max_output_bytes: 1,
                ..Limits::default()
            },
            Diagnostics::default(),
        )
        .unwrap();
    assert_eq!(r.output, [0]);
    assert_eq!(r.stop_reason, StopReason::OutputLimit);
    assert_eq!(r.instructions, 2);
    assert!(
        Program::parse(".")
            .unwrap()
            .execute(
                &[],
                Options::default(),
                Limits {
                    max_output_bytes: 1,
                    ..Limits::default()
                }
            )
            .is_ok()
    );
}
#[test]
fn trace_snapshots_and_serialized_report_budgets_are_explicit() {
    let p = Program::parse(" \n+>+<.").unwrap();
    let diag = Diagnostics {
        tape_start: 0,
        tape_len: 2,
        max_trace_records: 5,
        max_trace_bytes: bf::MAX_TRACE_BYTES,
        ..Diagnostics::default()
    };
    let a = p
        .report(&[], Options::default(), Limits::default(), diag)
        .unwrap();
    let b = p
        .report(&[], Options::default(), Limits::default(), diag)
        .unwrap();
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
    assert!(a.trace_complete);
    assert_eq!(a.trace.len(), 5);
    assert_eq!(a.trace[0].source_offset, 2);
    assert_eq!(a.trace[0].instruction, 1);
    assert_eq!(a.trace[0].cell_before, 0);
    assert_eq!(a.trace[0].cell_after, 1);
    assert_eq!(a.trace[1].pointer_before, 0);
    assert_eq!(a.trace[1].pointer_after, 1);
    assert_eq!(a.trace[1].cell_after, 0);
    assert_eq!(a.trace[4].output_byte, Some(1));
    assert_eq!(a.tape[1].index, 1);
    assert_eq!(a.tape[1].value, 1);
    assert!(serde_json::to_vec(&a).unwrap().len() <= a.report_bytes_upper_bound);
    for d in [
        Diagnostics {
            max_trace_records: 4,
            ..diag
        },
        Diagnostics {
            max_trace_bytes: 1,
            ..diag
        },
    ] {
        let r = p
            .report(&[], Options::default(), Limits::default(), d)
            .unwrap();
        assert!(r.completed);
        assert!(!r.trace_complete);
    }
    let limits = Limits {
        max_memory_bytes: a.memory_bytes_upper_bound,
        ..Limits::default()
    };
    let diag = Diagnostics {
        max_report_bytes: a.report_bytes_upper_bound,
        ..diag
    };
    assert!(p.report(&[], Options::default(), limits, diag).is_ok());
    assert!(
        p.report(
            &[],
            Options::default(),
            Limits {
                max_memory_bytes: limits.max_memory_bytes - 1,
                ..limits
            },
            diag
        )
        .is_err()
    );
    assert!(
        p.report(
            &[],
            Options::default(),
            limits,
            Diagnostics {
                max_report_bytes: diag.max_report_bytes - 1,
                ..diag
            }
        )
        .is_err()
    );
    for d in [
        Diagnostics {
            tape_start: 30000,
            tape_len: 1,
            ..diag
        },
        Diagnostics {
            tape_start: usize::MAX,
            tape_len: 1,
            ..diag
        },
        Diagnostics {
            max_trace_records: bf::MAX_TRACE_RECORDS + 1,
            ..diag
        },
        Diagnostics {
            max_trace_bytes: bf::MAX_TRACE_BYTES + 1,
            ..diag
        },
        Diagnostics {
            max_report_bytes: bf::MAX_REPORT_BYTES + 1,
            ..diag
        },
        Diagnostics {
            max_trace_records: 0,
            ..diag
        },
    ] {
        assert!(
            p.report(&[], Options::default(), Limits::default(), d)
                .is_err()
        );
    }
    let no_trace = p
        .report(
            &[],
            Options::default(),
            Limits::default(),
            Diagnostics {
                tape_start: 30000,
                tape_len: 0,
                ..Diagnostics::default()
            },
        )
        .unwrap();
    assert!(no_trace.trace.is_empty());
    assert!(no_trace.tape.is_empty());
}
#[test]
fn hard_limits_validate_before_runtime_allocation() {
    let p = Program::parse("").unwrap();
    for bits in [0, 7, 64, 255] {
        assert!(
            p.execute(
                &[],
                Options {
                    cell_bits: bits,
                    ..Options::default()
                },
                Limits::default()
            )
            .is_err()
        );
    }
    for l in [
        Limits {
            max_instructions: bf::MAX_INSTRUCTIONS + 1,
            ..Limits::default()
        },
        Limits {
            max_output_bytes: usize::MAX,
            ..Limits::default()
        },
        Limits {
            max_memory_bytes: usize::MAX,
            ..Limits::default()
        },
    ] {
        assert!(p.execute(&[], Options::default(), l).is_err());
    }
    assert!(
        p.execute(
            &vec![0; bf::MAX_INPUT_BYTES],
            Options::default(),
            Limits::default()
        )
        .is_ok()
    );
    assert_eq!(
        p.execute(
            &vec![0; bf::MAX_INPUT_BYTES + 1],
            Options::default(),
            Limits::default()
        )
        .unwrap_err()
        .code,
        ErrorCode::ResourceLimit
    );
    for id in [
        "esolang.brainfuck.execute",
        "esolang.brainfuck.report",
        "esolang.brainfuck.compare",
    ] {
        assert!(
            operations::require(id)
                .unwrap()
                .discovery()
                .pipeline
                .is_none()
        );
    }
}
/// An independent slow oracle searches brackets directly and uses u64 modulo;
/// the production parser/jump table and u32 wrapping are not reused.
fn oracle(source: &str, width: u32, budget: u64) -> (Vec<u8>, u64, bool) {
    let code = source.as_bytes();
    let mut tape = [0u64; 16];
    let mut pointer = 0;
    let mut pc = 0;
    let mut steps = 0;
    let mut output = Vec::new();
    while pc < code.len() && steps < budget {
        steps += 1;
        match code[pc] {
            b'+' => tape[pointer] = (tape[pointer] + 1) % (1u64 << width),
            b'-' => tape[pointer] = (tape[pointer] + (1u64 << width) - 1) % (1u64 << width),
            b'>' => pointer += 1,
            b'<' => pointer -= 1,
            b'.' => output.push((tape[pointer] % 256) as u8),
            b'[' if tape[pointer] == 0 => {
                let mut depth = 1;
                while depth > 0 {
                    pc += 1;
                    match code[pc] {
                        b'[' => depth += 1,
                        b']' => depth -= 1,
                        _ => (),
                    }
                }
            }
            b']' if tape[pointer] != 0 => {
                let mut depth = 1;
                while depth > 0 {
                    pc -= 1;
                    match code[pc] {
                        b']' => depth += 1,
                        b'[' => depth -= 1,
                        _ => (),
                    }
                }
            }
            _ => (),
        }
        pc += 1;
    }
    (output, steps, pc == code.len())
}
#[test]
fn width_sensitive_oracle_comparisons_and_aggregate_budgets() {
    let source = format!("{}[.-]", "+".repeat(256));
    let c = bf::compare(
        &source,
        &[],
        &[32, 8, 16],
        EofPolicy::Error,
        Limits::default(),
        ComparisonLimits::default(),
    )
    .unwrap();
    assert!(c.all_completed);
    assert_eq!(c.equal_completed_outputs, Some(false));
    for r in &c.runs {
        let (o, n, complete) = oracle(
            &source,
            u32::from(r.options.cell_bits),
            bf::MAX_INSTRUCTIONS,
        );
        assert_eq!(r.output, o);
        assert_eq!(r.instructions, n);
        assert_eq!(r.completed, complete);
    }
    assert_eq!(c.runs[0].options.cell_bits, 8);
    assert_eq!(c.runs[1].options.cell_bits, 16);
    let c = bf::compare(
        "-.",
        &[],
        &[8, 16, 32],
        EofPolicy::Error,
        Limits::default(),
        ComparisonLimits::default(),
    )
    .unwrap();
    assert_eq!(c.equal_completed_outputs, Some(true));
    let c = bf::compare(
        "+[]",
        &[],
        &[8, 16],
        EofPolicy::Error,
        Limits {
            max_instructions: 5,
            ..Limits::default()
        },
        ComparisonLimits {
            max_instructions: 7,
            ..ComparisonLimits::default()
        },
    )
    .unwrap();
    assert_eq!(c.instructions, 7);
    assert_eq!(c.runs[1].instructions, 2);
    assert_eq!(c.equal_completed_outputs, None);
    assert!(!c.all_completed);
    let c = bf::compare(
        "..",
        &[],
        &[8, 16],
        EofPolicy::Error,
        Limits::default(),
        ComparisonLimits {
            max_output_bytes: 3,
            ..ComparisonLimits::default()
        },
    )
    .unwrap();
    assert_eq!(c.output_bytes, 3);
    assert!(!c.all_completed);
    assert_eq!(c.runs[1].stop_reason, StopReason::OutputLimit);
    for widths in [&[][..], &[8, 8], &[7], &[8, 16, 32, 8]] {
        assert!(
            bf::compare(
                "",
                &[],
                widths,
                EofPolicy::Error,
                Limits::default(),
                ComparisonLimits::default()
            )
            .is_err()
        );
    }
    for a in [
        ComparisonLimits {
            max_instructions: bf::MAX_INSTRUCTIONS + 1,
            ..ComparisonLimits::default()
        },
        ComparisonLimits {
            max_output_bytes: usize::MAX,
            ..ComparisonLimits::default()
        },
        ComparisonLimits {
            max_memory_bytes: 0,
            ..ComparisonLimits::default()
        },
        ComparisonLimits {
            max_memory_bytes: usize::MAX,
            ..ComparisonLimits::default()
        },
    ] {
        assert!(bf::compare("", &[], &[8], EofPolicy::Error, Limits::default(), a).is_err());
    }
}

#[test]
fn aggregate_memory_and_trace_byte_edges_and_stop_provenance() {
    use bf::BudgetScope;
    let c = bf::compare(
        "+[]",
        &[],
        &[8, 16],
        EofPolicy::Error,
        Limits {
            max_instructions: 5,
            ..Limits::default()
        },
        ComparisonLimits {
            max_instructions: 7,
            ..ComparisonLimits::default()
        },
    )
    .unwrap();
    assert_eq!(c.runs[0].budget_scope, Some(BudgetScope::PerRun));
    assert_eq!(c.runs[1].budget_scope, Some(BudgetScope::Aggregate));
    let c = bf::compare(
        "+[]",
        &[],
        &[8],
        EofPolicy::Error,
        Limits {
            max_instructions: 5,
            ..Limits::default()
        },
        ComparisonLimits {
            max_instructions: 5,
            ..ComparisonLimits::default()
        },
    )
    .unwrap();
    assert_eq!(c.runs[0].budget_scope, Some(BudgetScope::Both));
    let memory = c.memory_bytes_upper_bound;
    assert!(
        bf::compare(
            "+[]",
            &[],
            &[8],
            EofPolicy::Error,
            Limits {
                max_instructions: 5,
                ..Limits::default()
            },
            ComparisonLimits {
                max_instructions: 5,
                max_memory_bytes: memory,
                ..ComparisonLimits::default()
            }
        )
        .is_ok()
    );
    assert!(
        bf::compare(
            "+[]",
            &[],
            &[8],
            EofPolicy::Error,
            Limits {
                max_instructions: 5,
                ..Limits::default()
            },
            ComparisonLimits {
                max_instructions: 5,
                max_memory_bytes: memory - 1,
                ..ComparisonLimits::default()
            }
        )
        .is_err()
    );
    let p = Program::parse("++").unwrap();
    let bytes = std::mem::size_of::<bf::TraceRecord>() * 2;
    for (limit, records, complete) in [(bytes, 2, true), (bytes - 1, 1, false)] {
        let r = p
            .report(
                &[],
                Options::default(),
                Limits::default(),
                Diagnostics {
                    max_trace_records: 2,
                    max_trace_bytes: limit,
                    ..Diagnostics::default()
                },
            )
            .unwrap();
        assert_eq!(r.trace.len(), records);
        assert_eq!(r.trace_complete, complete);
    }
    let r = report("+[]", 8, bf::MAX_INSTRUCTIONS);
    assert_eq!(r.instructions, bf::MAX_INSTRUCTIONS);
    assert!(!r.completed);
}

#[test]
fn exact_full_tape_and_last_cell_snapshots_preserve_values() {
    let p = Program::parse(&format!("{}+", ">".repeat(29999))).unwrap();
    let r = p
        .report(
            &[],
            Options {
                cell_bits: 32,
                ..Options::default()
            },
            Limits::default(),
            Diagnostics {
                tape_start: 29999,
                tape_len: 1,
                ..Diagnostics::default()
            },
        )
        .unwrap();
    assert!(r.completed);
    assert_eq!(r.tape[0].index, 29999);
    assert_eq!(r.tape[0].value, 1);
    let r = Program::parse("")
        .unwrap()
        .report(
            &[],
            Options::default(),
            Limits::default(),
            Diagnostics {
                tape_len: bf::TAPE_CELLS,
                ..Diagnostics::default()
            },
        )
        .unwrap();
    assert_eq!(r.tape.len(), bf::TAPE_CELLS);
    assert_eq!(r.tape.last().unwrap().index, 29999);
    assert!(r.tape.iter().all(|c| c.value == 0));
    assert!(serde_json::to_vec(&r).unwrap().len() <= r.report_bytes_upper_bound);
}
