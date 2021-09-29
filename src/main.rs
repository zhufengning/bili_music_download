use bili_music_download::bapi;
use iced::{
    button, scrollable, text_input, Application, Button, Checkbox, Clipboard, Column, Command,
    Container, Element, Length, Row, Scrollable, Settings, Text, TextInput,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> iced::Result {
    let mut my_settings = Settings::default();
    my_settings.default_font = Some(include_bytes!("LXGWWenKai-Regular.ttf"));
    my_settings.window.size = (300, 200);
    return App::run(my_settings);
}

#[derive(Clone, Debug)]
enum Message {
    QrLoginPressed,
    CookieLoginPressed,
    FinishCookieLogin,
    CookieInputChanged(String),
    FavInputChanged(String),
    GetList,
    GotList(Option<Vec<serde_json::Value>>),
    Check(usize, CheckMessage),
    SelectAll,
    Fanxuan,
    Clear,
    FinalStep,
    ChooseFile,
    StartDown,
    Finish(bool),
    ChangePath(String),
    ChangeProg(f64),
}

enum Pages {
    Login,
    CookieLogin,
    ListPage,
    SavePage,
}

struct App {
    page: Pages,
    qr_login_button: button::State,
    cookie_login_button: button::State,
    select_all_button: button::State,
    fanxuan_button: button::State,
    clear_select_button: button::State,
    final_step_button: button::State,
    choose_file_button: button::State,
    start_download_button: button::State,
    scroll: scrollable::State,
    cookie_input: text_input::State,
    fav_id_input: text_input::State,
    do_cookie_login_button: button::State,
    cookie_value: &'static str,
    cookie_placeholder: String,
    fav_id_value: &'static str,
    fav_id_placeholder: String,
    get_list_button: button::State,
    fav_list: Vec<serde_json::Value>,
    down_list: &'static Vec<serde_json::Value>,
    fav_list_v: HashMap<usize, bool>,
    fav_lists: Vec<Check>,
    msg: String,
    path: &'static str,
    progress: &'static Arc<Mutex<f64>>,
    downloading: bool,
    prog_percent: f64,
    start_down_msg: String,
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();
    fn new(_: ()) -> (App, Command<Message>) {
        (
            App {
                page: Pages::Login,
                qr_login_button: button::State::new(),
                cookie_login_button: button::State::new(),
                select_all_button: button::State::new(),
                fanxuan_button: button::State::new(),
                clear_select_button: button::State::new(),
                final_step_button: button::State::new(),
                choose_file_button: button::State::new(),
                start_download_button: button::State::new(),
                scroll: scrollable::State::new(),
                cookie_input: text_input::State::new(),
                fav_id_input: text_input::State::new(),
                do_cookie_login_button: button::State::new(),
                cookie_value: "",
                cookie_placeholder: String::from("输入SESSDATA"),
                fav_id_value: "",
                fav_id_placeholder: String::from("输入收藏夹编号"),
                get_list_button: button::State::default(),
                fav_list: vec![],
                down_list: Box::leak(Vec::new().into()),
                fav_list_v: HashMap::new(),
                fav_lists: vec![],
                msg: "".to_string(),
                path: "",
                progress: Box::leak(Arc::new(Mutex::new(0.0)).into()),
                downloading: false,
                prog_percent: 0.,
                start_down_msg: String::from("开始下载"),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Bili音乐下载")
    }

    fn update(&mut self, message: Self::Message, _: &mut Clipboard) -> Command<Message> {
        match message {
            Message::QrLoginPressed => Command::none(),
            Message::CookieLoginPressed => {
                self.page = Pages::CookieLogin;
                Command::none()
            }
            Message::CookieInputChanged(s) => {
                self.cookie_value = Box::leak(s.into_boxed_str());
                Command::none()
            }
            Message::FinishCookieLogin => {
                self.page = Pages::ListPage;
                Command::none()
            }
            Message::FavInputChanged(s) => {
                self.fav_id_value = Box::leak(s.into_boxed_str());
                Command::none()
            }
            Message::GetList => {
                self.msg = String::from("获取列表...\n（不要重复按OK）");
                Command::perform(
                    get_video_list(self.fav_id_value, self.cookie_value),
                    Message::GotList,
                )
            }
            Message::GotList(l) => {
                self.msg = String::from("获取完成\n把鼠标移到OK下面即可看到滚动条");
                if let Some(l) = l {
                    self.fav_list_v = HashMap::new();
                    self.fav_lists = vec![];
                    l.iter().enumerate().for_each(|(i, v)| {
                        self.fav_list_v.insert(i, true);
                        self.fav_lists.push(Check::new(
                            v["title"]
                                .as_str()
                                .unwrap_or_default()
                                .parse()
                                .unwrap_or_default(),
                        ));
                    });
                    self.fav_list = l;
                }
                Command::none()
            }
            Message::Check(i, v) => {
                self.fav_lists[i].update(v);
                Command::none()
            }
            Message::SelectAll => {
                self.fav_lists.iter_mut().for_each(|v| v.v = true);
                Command::none()
            }
            Message::Clear => {
                self.fav_lists.iter_mut().for_each(|v| v.v = false);
                Command::none()
            }
            Message::Fanxuan => {
                self.fav_lists.iter_mut().for_each(|v| v.v = !v.v);
                Command::none()
            }
            Message::FinalStep => {
                let mut down_list: Vec<serde_json::Value> = vec![];
                self.fav_list.iter().enumerate().for_each(|(i, v)| {
                    if self.fav_list_v[&i] == true {
                        down_list.push(v.clone());
                    }
                });
                self.down_list = Box::leak(down_list.into());
                self.msg = "".into();
                self.page = Pages::SavePage;
                Command::none()
            }
            Message::ChooseFile => Command::perform(choose_file(), Message::ChangePath),
            Message::StartDown => {
                if !self.downloading {
                    self.downloading = true;
                    self.msg = String::from("下载中...但进度条不会自己动");
                    self.start_down_msg = String::from("刷新进度条");
                    Command::perform(
                        start_download(
                            &self.down_list,
                            self.cookie_value,
                            &self.path,
                            Arc::clone(self.progress),
                        ),
                        Message::Finish,
                    )
                } else {
                    Command::perform(open_mu(self.progress), Message::ChangeProg)
                }
            }
            Message::Finish(_) => {
                self.msg = String::from("下载完成！");
                Command::none()
            }
            Message::ChangePath(u) => {
                self.path = Box::leak(u.as_str().into());
                Command::none()
            }
            Message::ChangeProg(p) => {
                self.prog_percent = p;
                Command::none()
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let this_page: Element<_> = match self.page {
            Pages::Login => Row::new()
                .spacing(20)
                .push(
                    Button::new(&mut self.qr_login_button, Text::new("二维码登录（没做）"))
                        .on_press(Message::QrLoginPressed),
                )
                .push(
                    Button::new(&mut self.cookie_login_button, Text::new("Cookie登录"))
                        .on_press(Message::CookieLoginPressed),
                )
                .into(),
            Pages::CookieLogin => Row::new()
                .spacing(20)
                .push(TextInput::new(
                    &mut self.cookie_input,
                    &self.cookie_placeholder,
                    &self.cookie_value,
                    Message::CookieInputChanged,
                ))
                .push(
                    Button::new(&mut self.do_cookie_login_button, Text::new("登录"))
                        .on_press(Message::FinishCookieLogin),
                )
                .into(),
            Pages::ListPage => {
                let res = Column::new()
                    .push(
                        Row::new()
                            .push(TextInput::new(
                                &mut self.fav_id_input,
                                &self.fav_id_placeholder,
                                &self.fav_id_value,
                                Message::FavInputChanged,
                            ))
                            .push(
                                Button::new(&mut self.get_list_button, Text::new("OK"))
                                    .on_press(Message::GetList),
                            ),
                    )
                    .push(Text::new(&self.msg));
                let len = self.fav_list.len().clone();
                let list = Column::new().push(self.fav_lists.iter_mut().enumerate().fold(
                    Column::new().spacing(5),
                    |col, (index, check)| {
                        // We display the counter
                        let element: Element<CheckMessage> = check.view().into();

                        col.push(Text::new(format!("{}", index))).push(
                            // Here we turn our `Element<counter::Message>` into
                            // an `Element<Message>` by combining the `index` and the
                            // message of the `element`.
                            element.map(move |message| Message::Check(index, message)),
                        )
                    },
                ));
                let mut options = Row::new();
                if len > 0 {
                    options = options
                        .push(
                            Button::new(&mut self.select_all_button, Text::new("全选"))
                                .on_press(Message::SelectAll),
                        )
                        .push(
                            Button::new(&mut self.fanxuan_button, Text::new("反选"))
                                .on_press(Message::Fanxuan),
                        )
                        .push(
                            Button::new(&mut self.clear_select_button, Text::new("全不选"))
                                .on_press(Message::Clear),
                        )
                        .push(
                            Button::new(&mut self.final_step_button, Text::new("下一步"))
                                .on_press(Message::FinalStep),
                        );
                }
                res.push(options).push(list).into()
            }
            Pages::SavePage => Column::new()
                .push(Text::new(self.path.clone()))
                .push(
                    Button::new(&mut self.choose_file_button, Text::new("浏览..."))
                        .on_press(Message::ChooseFile),
                )
                .push(
                    Button::new(
                        &mut self.start_download_button,
                        Text::new(&self.start_down_msg),
                    )
                    .on_press(Message::StartDown),
                )
                .push(Text::new(&self.msg))
                .push(Text::new(format!(
                    "进度：{:.3}%",
                    100. * self.prog_percent / self.down_list.len() as f64
                )))
                .into(),
        };
        let scrollable = Scrollable::new(&mut self.scroll)
            .push(Container::new(this_page).width(Length::Fill).center_x());

        Container::new(scrollable)
            .height(Length::Fill)
            .width(Length::FillPortion(10))
            .center_y()
            .into()
    }
}
#[derive(Clone, Debug)]
enum CheckMessage {
    Check(bool),
}

struct Check {
    title: String,
    v: bool,
}

impl Check {
    fn new(title: String) -> Self {
        Check { title, v: true }
    }

    fn update(&mut self, message: CheckMessage) {
        match message {
            CheckMessage::Check(v) => {
                self.v = v;
            }
        }
    }

    fn view(&mut self) -> Element<CheckMessage> {
        Checkbox::new(self.v, &self.title, CheckMessage::Check).into()
    }
}

async fn open_mu(m: &Arc<Mutex<f64>>) -> f64 {
    let prog = *Arc::clone(m).lock().await;
    prog
}

async fn get_video_list(fid: &str, sessdata: &str) -> Option<Vec<serde_json::Value>> {
    let v_list = bapi::get_fav_list(fid, sessdata).await;
    match v_list {
        Ok(v) => Some(v),
        Err(e) => {
            println!("{:?}", e);
            None
        }
    }
}

async fn add_prog(prog: Arc<Mutex<f64>>) {
    let mut prog = prog.lock().await;
    *prog += 1.;
}

async fn start_download(
    v_list: &Vec<serde_json::Value>,
    sessdata: &str,
    path: &str,
    prog: Arc<Mutex<f64>>,
) -> bool {
    println!("共{}项", v_list.len());
    let mut cnt1: i32 = 0;
    for e in v_list.iter() {
        cnt1 += 1;
        println!("第{}个视频", cnt1);
        let bvid = e["bvid"].as_str().unwrap_or_default();
        let video_inf: bapi::VideoInf = bapi::VideoInf {
            author: e["upper"]["name"]
                .as_str()
                .unwrap_or_default()
                .parse()
                .unwrap_or_default(),
            name: e["title"]
                .as_str()
                .unwrap_or_default()
                .parse()
                .unwrap_or_default(),
        };
        let ps = bapi::get_ps(bvid, sessdata).await;
        match ps {
            Ok(ps) => {
                let ps = ps["data"].as_array();
                match ps {
                    Some(ps) => {
                        let mut cnt2 = 0;
                        for p in ps.iter() {
                            cnt2 += 1;
                            println!("第{}P", cnt2);
                            let cid = format!("{}", p["cid"].as_i64().unwrap_or_default());
                            let p_name = p["part"].as_str().unwrap_or_default();
                            let mut file_name =
                                format!("{} - {} - {}", video_inf.name, p_name, video_inf.author);
                            file_name = file_name.replace("\\", " ");
                            file_name = file_name.replace("/", " ");
                            file_name = file_name.replace("?", " ");
                            file_name = file_name.replace("*", " ");
                            file_name = file_name.replace(">", " ");
                            file_name = file_name.replace("<", " ");
                            file_name = file_name.replace("|", " ");
                            file_name = file_name.replace(":", " ");
                            println!("{}", file_name);
                            match bapi::get_url(bvid, cid.as_str(), sessdata).await {
                                Ok(u) => match u["data"]["dash"]["audio"].as_array() {
                                    Some(u) => {
                                        let music_url =
                                            u[0]["baseUrl"].as_str().unwrap_or_default();
                                        match bapi::download_music(
                                            &format!("{}/{}.aac", path, file_name),
                                            music_url,
                                            sessdata,
                                        )
                                        .await
                                        {
                                            Ok(_) => {
                                            }
                                            Err(e) => {
                                                println!("{:?}", e);
                                            }
                                        }
                                    }
                                    None => {}
                                },
                                Err(e) => {
                                    println!("{:?}", e);
                                }
                            }
                        }
                    }
                    None => {}
                }
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
        add_prog(Arc::clone(&prog)).await;
        println!("---------");
    }
    return true;
}

async fn choose_file() -> String {
    let r = rfd::AsyncFileDialog::new().pick_folder().await;
    if let Some(u) = r {
        return u.path().to_str().unwrap_or_default().into();
    }
    return String::new();
}
