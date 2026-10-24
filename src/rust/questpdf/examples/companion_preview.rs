use questpdf::companion::{CompanionOptions, CompanionSession};
use questpdf::*;
fn main() -> anyhow::Result<()> {
    Settings::set_license(LicenseType::Community);
    let args: Vec<_> = std::env::args().collect();
    let url = args
        .get(1)
        .cloned()
        .unwrap_or("http://127.0.0.1:12500".into());
    let invalid = args.get(2).is_some_and(|x| x == "invalid");
    let document = Document::create(|d| {
        d.page(|p| {
            p.size(Size::new(240., 180.));
            p.margin(10.);
            p.default_text_style(|s| s.font_size(10.));
            if invalid {
                p.content().width(400.).text("Cannot fit");
            } else {
                p.content().column(|c| {
                    c.item()
                        .debug_pointer("Preview fixture")
                        .border(1.)
                        .padding(4.)
                        .text("Companion API v3");
                    c.item().page_break();
                    c.item().background(Color(0xffbbddff)).text("Second page");
                });
                p.footer().text_spans(|t| {
                    t.span("Page ");
                    t.current_page_number();
                });
            }
        })
    });
    let mut session = CompanionSession::connect(&CompanionOptions {
        base_url: Some(url),
        launch_application: false,
        ..Default::default()
    })?;
    let mut renderer = Renderer::new()?;
    if invalid {
        assert!(
            session
                .update_document(&mut renderer, &document, false)
                .is_err()
        );
        return Ok(());
    }
    session.update_document(&mut renderer, &document, false)?;
    drop(renderer); // Native snapshots must retain their own text/image resources.
    if args.get(2).is_some_and(|x| x == "run") {
        let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let signal = cancelled.clone();
        let worker = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(700));
            signal.store(true, std::sync::atomic::Ordering::Release);
        });
        session.run(&cancelled)?;
        worker.join().unwrap();
        return Ok(());
    }
    for _ in 0..4 {
        assert!(session.poll()?);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    if args.get(2).is_some_and(|x| x == "refresh") {
        session.update_document(&mut Renderer::new()?, &document, true)?;
        assert!(session.poll()?);
    }
    Ok(())
}
