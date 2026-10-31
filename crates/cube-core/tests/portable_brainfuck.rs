use cube_core::{
    errors::ErrorCode,
    esolang::brainfuck::{Diagnostics, Limits, Options, Program},
};
#[test]
fn trace_and_tape_reservations_have_portable_exact_boundaries() {
    let program = Program::parse(",.").unwrap();
    let limits = Limits {
        max_output_bytes: 1,
        max_memory_bytes: 126976,
        ..Limits::default()
    };
    let diagnostics = Diagnostics {
        max_trace_records: 2,
        max_trace_bytes: 96,
        tape_len: 1,
        max_report_bytes: 4788,
        ..Diagnostics::default()
    };
    let report = program
        .report(&[255], Options::default(), limits, diagnostics)
        .unwrap();
    assert_eq!(report.memory_bytes_upper_bound, 126976);
    assert_eq!(report.report_bytes_upper_bound, 4788);
    assert_eq!(report.trace.len(), 2);
    assert_eq!(report.tape[0].value, 255);
    let error = program
        .report(
            &[255],
            Options::default(),
            Limits {
                max_memory_bytes: 126975,
                ..limits
            },
            diagnostics,
        )
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::ResourceLimit);
}
