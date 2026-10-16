use skein_lib::Rng;
use temper_codecs_world::{forgejo, mutated, mutated_into};
#[test]
fn forgejo_decoders_answer_every_mutated_transcript_without_panicking() {
    let mut rng = Rng::new(0x06_05);
    for (case, request) in forgejo::json_cases() {
        let body = forgejo::transcript(&case).body;
        let mut bytes = Vec::new();
        for _ in 0..200 {
            mutated_into(&mut rng, &body, &mut bytes);
            let decoded = match &request {
                Some(request) => forgejo::decode(request, &bytes),
                None => forgejo::decode_error(&bytes),
            };
            assert!(decoded.steps <= forgejo::STEP_BOUND, "{case}: decode terminates within its bound");
        }
    }
}
#[test]
fn forgejo_server_routes_answer_every_mutated_request_without_panicking() {
    let mut rng = Rng::new(0x06_05_02);
    let limits = forgejo::limits();
    for case in forgejo::transcript_cases() {
        let transcript = forgejo::transcript(&case);
        for _ in 0..200 {
            let target = mutated(&mut rng, &transcript.target);
            let body = mutated(&mut rng, &transcript.request_body);
            if let Ok((_, steps)) = forgejo::decode_route(transcript.method, &target, &body, &limits) {
                assert!(steps <= forgejo::STEP_BOUND, "{case}: route terminates within its bound");
            }
        }
    }
}
