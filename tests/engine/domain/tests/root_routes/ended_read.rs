use super::*;
const MARK: Token = Token::new(1201);
const READ: Token = Token::new(1202);

fn ended_chat() -> Store {
    let (mut driver, assignment) = Driver::chat();
    for number in 1..=2 {
        turn(&mut driver, &assignment, number, u64::from(number));
        driver.settle();
        if number == 1 {
            driver.send(engine::Event::Party(engine::PartyInput::Ask {
                reply_to: ReplyTo::new(MARK),
                sign_in: driver.session(),
                key: [32; 16],
                ask: people::Ask::MarkSeen { seen: people::Seen::Chat { task: assignment.task, through: 1 } },
            }));
            driver.settle();
        }
    }
    driver.answer(
        Token::new(7),
        assignment.task,
        assignment.attempt,
        2,
        tasks::End::Finished { result: tasks::TaskResult::Report { words: REPORT.into() }, cancel_delegates: false },
        engine::Parts::none(),
    );
    driver.settle();
    driver.store
}

#[test]
fn an_ended_chats_read_position_survives_a_restart() {
    let mut restarted = Driver::new(ended_chat());
    restarted.settle();
    restarted.sign_in();
    restarted.settle();
    let line = conversation(&mut restarted);
    assert_eq!((line.seen, line.last_reply), (1, 2));
    assert_eq!(line.replies_unread, Some(jig_domain_core::documents::Count::Known(1)));
}

fn conversation(driver: &mut Driver) -> jig_domain_core::documents::ConversationLine {
    use jig_domain_core::reads;
    let sign_in = driver.session();
    let mut from = None;
    loop {
        driver.delivered.clear();
        driver.send(engine::Event::Party(engine::PartyInput::Read {
            reply_to: ReplyTo::new(READ),
            sign_in,
            read: reads::Read::Conversations { from },
        }));
        driver.settle();
        let page = driver
            .delivered
            .iter()
            .find_map(|delivery| {
                if let Delivery::Read { answer: reads::Answer::Page(page), .. } = delivery { Some(page) } else { None }
            })
            .expect("conversation page");
        if let Some(line) = page
            .rows
            .iter()
            .find_map(|row| if let reads::PageRow::Conversation(line) = row { Some(line.clone()) } else { None })
        {
            return line;
        }
        from = Some(page.next.clone().expect("ended conversation continuation"));
    }
}
