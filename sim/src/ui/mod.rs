use gtk::{glib, prelude::*};
use std::{
    cell::RefCell,
    rc::Rc,
    time::Duration
};
use stcore::{Command, Core, State, STACKSIZE};

const CSS: &str = "
.sp   { background: #f9a825; color: black; font-weight: bold; }
.free { color: #888; }
.stage-on { background: #2e7d32; color: white; font-weight: bold; }
.mono { font-family: monospace; }
";

struct View {
    scroll: gtk::ScrolledWindow,
    idx: Vec<gtk::Label>,
    bin: Vec<gtk::Label>,
    dec: Vec<gtk::Label>,
    hex: Vec<gtk::Label>,
    regs: Vec<(&'static str, gtk::Label)>,
    stages: Vec<gtk::Label>,
    alu: gtk::Label,
}

fn label(text: &str) -> gtk::Label {
    let l = gtk::Label::new(Some(text));
    l.add_css_class("mono");
    l.set_xalign(0.0);
    l
}

impl View {
    pub fn new() -> (Self, gtk::Box) {
        let root = gtk::Box::new(gtk::Orientation::Horizontal, 12);

        let grid = gtk::Grid::builder().column_spacing(16).build();
        for (c, h) in ["#", "bin", "dec", "hex"].iter().enumerate() {
            grid.attach(&label(h), c as i32, 0, 1, 1);
        }
        let (mut idx, mut bin, mut dec, mut hex) = (vec![], vec![], vec![], vec![]);
        for i in 0..STACKSIZE {
            let (a, b, c, d) = (label(&i.to_string()), label(""), label(""), label(""));
            grid.attach(&a, 0, i as i32 + 1, 1, 1);
            grid.attach(&b, 1, i as i32 + 1, 1, 1);
            grid.attach(&c, 2, i as i32 + 1, 1, 1);
            grid.attach(&d, 3, i as i32 + 1, 1, 1);
            idx.push(a); bin.push(b); dec.push(c); hex.push(d);
        }
        let scroll = gtk::ScrolledWindow::builder()
            .child(&grid).min_content_width(220).vexpand(true).build();
        root.append(&scroll);

        let right = gtk::Box::new(gtk::Orientation::Vertical, 8);

        let regs_grid = gtk::Grid::builder().column_spacing(12).row_spacing(2).build();
        let names = ["pc", "sp", "cnt", "cmd", "addr", "lit", "op1", "op2", "op3", "res", "fZ", "fN", "fV"];
        let mut regs = vec![];
        for (i, n) in names.iter().enumerate() {
            let v = label("");
            regs_grid.attach(&label(n), 0, i as i32, 1, 1);
            regs_grid.attach(&v, 1, i as i32, 1, 1);
            regs.push((*n, v));
        }
        let f1 = gtk::Frame::new(Some("Регистры ядра"));
        f1.set_child(Some(&regs_grid));
        right.append(&f1);

        let st_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        let stages: Vec<_> = ["FETCH", "DECODE", "EXECUTE", "WRITE"]
            .iter().map(|s| { let l = label(s); l.set_margin_start(6); l.set_margin_end(6); st_box.append(&l); l })
            .collect();
        let f2 = gtk::Frame::new(Some("Стадия"));
        f2.set_child(Some(&st_box));
        right.append(&f2);

        let alu = label("");
        let f3 = gtk::Frame::new(Some("АЛУ"));
        f3.set_child(Some(&alu));
        right.append(&f3);

        root.append(&right);
        (View { scroll, idx, bin, dec, hex, regs, stages, alu }, root)
    }

    fn update(&self, c: &Core) {
        for i in 0..STACKSIZE {
            let v = c.stack[i];
            self.bin[i].set_text(&format!("{:08b}", v as i8));
            self.dec[i].set_text(&v.to_string());
            self.hex[i].set_text(&format!("{:02X}", v as u8));
            let is_sp = i as isize == c.sp;
            let free = i as isize > c.sp;
            for l in [&self.idx[i], &self.bin[i], &self.dec[i], &self.hex[i]] {
                if is_sp { l.add_css_class("sp") } else { l.remove_css_class("sp") }
                if free { l.add_css_class("free") } else { l.remove_css_class("free") }
            }
        }

        if c.sp >= 0 {
            let row = self.idx[c.sp as usize].clone();
            let scroll = self.scroll.clone();
            glib::idle_add_local_once(move || {
                let adj = scroll.vadjustment();
                let y = row.allocation().y() as f64;
                let target = y - adj.page_size() / 2.0;
                adj.set_value(target.clamp(adj.lower(), (adj.upper() - adj.page_size()).max(0.0)));
            });
        }

        for (n, l) in &self.regs {
            let t = match *n {
                "pc" => c.pc.to_string(),
                "sp" => c.sp.to_string(),
                "cnt" => c.cnt.to_string(),
                "cmd" => format!("{:?}", c.cmd),
                "addr" => c.addr.to_string(),
                "lit" => c.lit.to_string(),
                "op1" => c.op1.to_string(),
                "op2" => c.op2.to_string(),
                "op3" => c.op3.to_string(),
                "fZ" => c.flags.z.to_string(),
                "fN" => c.flags.n.to_string(),
                "fV" => c.flags.v.to_string(),
                _ => c.res.to_string(),
            };
            l.set_text(&t);
        }

        let cur = match c.state { State::FETCH => 0, State::DECODE => 1, State::EXECUTE => 2, State::WRITE => 3 };
        for (i, l) in self.stages.iter().enumerate() {
            if i == cur { l.add_css_class("stage-on") } else { l.remove_css_class("stage-on") }
        }
        
        let t = match c.cmd {
            Command::ADD => format!("{} + {} = {}", c.op2, c.op1, c.res),
            Command::MUL => format!("{} * {} = {}", c.op2, c.op1, c.res),
            Command::INC => format!("{} + 1 = {}", c.op1, c.res),
            Command::LOOP => format!("cnt: {} - 1 = {}   ({})", c.op1, c.res,
                                     if c.res != 0 { "переход" } else { "выход" }),
            Command::READ => format!("mem[{}] = {}", c.op1, c.res),
            Command::DUP => format!("res = op1 = {}", c.res),
            _ => "—".into(),
        };
        self.alu.set_text(&t);
    }
}

pub fn build_ui(app: &gtk::Application) {
    let css = gtk::CssProvider::new();
    css.load_from_data(CSS);
    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().unwrap(), &css, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

    let core = Rc::new(RefCell::new(Core::new()));
    let prog = Rc::new(RefCell::new(Vec::<i8>::new()));
    let mem  = Rc::new(RefCell::new(Vec::<i8>::new()));
    let timer: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));

    let (view, body) = View::new();
    let view = Rc::new(view);
    view.update(&core.borrow());

    let bar = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let b_load = gtk::Button::with_label("Загрузить");
    let b_step = gtk::Button::with_label("Стадия");
    let b_cmd  = gtk::Button::with_label("Команда");
    let b_run  = gtk::ToggleButton::with_label("Пуск");
    let b_rst  = gtk::Button::with_label("Сброс");
    for b in [b_load.upcast_ref::<gtk::Widget>(), b_step.upcast_ref(), b_cmd.upcast_ref(),
              b_run.upcast_ref(), b_rst.upcast_ref()] { bar.append(b); }

    {
        let (core, prog, mem, view) = (core.clone(), prog.clone(), mem.clone(), view.clone());
        b_step.connect_clicked(move |_| {
            core.borrow_mut().step(&prog.borrow(), &mem.borrow());
            view.update(&core.borrow());
        });
    }

    {
        let (core, prog, mem, view) = (core.clone(), prog.clone(), mem.clone(), view.clone());
        b_cmd.connect_clicked(move |_| {
            core.borrow_mut().next(&prog.borrow(), &mem.borrow());
            view.update(&core.borrow());
        });
    }

    {
        let (core, prog, mem, view, timer) =
            (core.clone(), prog.clone(), mem.clone(), view.clone(), timer.clone());
        b_run.connect_toggled(move |btn| {
            if btn.is_active() {
                let (core, prog, mem, view, timer2, btn) =
                    (core.clone(), prog.clone(), mem.clone(), view.clone(), timer.clone(), btn.clone());
                let id = glib::timeout_add_local(Duration::from_millis(50), move || {
                    if core.borrow().halt {
                        timer2.borrow_mut().take();
                        btn.set_active(false);
                        return glib::ControlFlow::Break;
                    }
                    core.borrow_mut().next(&prog.borrow(), &mem.borrow());
                    view.update(&core.borrow());
                    glib::ControlFlow::Continue
                });
                *timer.borrow_mut() = Some(id);
            } else if let Some(id) = timer.borrow_mut().take() {
                id.remove();
            }
        });
    }

    {
        let (core, view, b_run) = (core.clone(), view.clone(), b_run.clone());
        b_rst.connect_clicked(move |_| {
            b_run.set_active(false);
            core.borrow_mut().reset();
            view.update(&core.borrow());
        });
    }

    {
        let (core, prog, mem, view) = (core.clone(), prog.clone(), mem.clone(), view.clone());
        let app = app.clone();
        b_load.connect_clicked(move |_| {
            let dlg = gtk::FileDialog::new();
            let (core, prog, mem, view) = (core.clone(), prog.clone(), mem.clone(), view.clone());
            dlg.open(app.active_window().as_ref(), gtk::gio::Cancellable::NONE, move |res| {
                if let Ok(file) = res {
                    if let Some(path) = file.path() {
                        if let Ok(fw) = std::fs::read(path) {
                            let sp = 384.min(fw.len());
                            *prog.borrow_mut() = fw[..sp].iter().map(|&b| b as i8).collect();
                            *mem.borrow_mut()  = fw[sp..].iter().map(|&b| b as i8).collect();
                            core.borrow_mut().reset();
                            view.update(&core.borrow());
                        }
                    }
                }
            });
        });
    }

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 8);
    vbox.set_margin_top(8); vbox.set_margin_bottom(8);
    vbox.set_margin_start(8); vbox.set_margin_end(8);
    vbox.append(&bar);
    body.set_vexpand(true);
    vbox.append(&body);

    gtk::ApplicationWindow::builder()
        .application(app).title("Стековый процессор")
        .default_width(620).default_height(560).child(&vbox).build()
        .present();
}
