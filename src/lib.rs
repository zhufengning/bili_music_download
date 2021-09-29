pub mod bapi {
    use std::io::Write;

    #[derive(Clone, Debug)]
    pub struct VideoInf {
        pub name: String,
        pub author: String,
    }

    impl Default for VideoInf {
        fn default() -> VideoInf {
            VideoInf {
                name: String::from("What?"),
                author: String::from("Who?"),
            }
        }
    }

    #[derive(Debug)]
    pub struct BError {
        code: i64,
        msg: String,
    }

    impl std::fmt::Display for BError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}:{}", self.code, self.msg)
        }
    }

    #[derive(Debug)]
    pub enum MyError {
        ReqError(reqwest::Error),
        BiliError(BError),
    }

    impl std::error::Error for BError {}

    impl std::convert::From<reqwest::Error> for MyError {
        fn from(r: reqwest::Error) -> MyError {
            MyError::ReqError(r)
        }
    }

    fn gen_cookie(sessdata: &str) -> String {
        format!("sessdata={}", urlencoding::encode(sessdata))
    }

    pub async fn get_fav_list(
        fid: &str,
        sessdata: &str,
    ) -> Result<Vec<serde_json::Value>, MyError> {
        let mut has_more = true;
        let mut list: Vec<serde_json::Value> = vec![];
        let mut pn = 1;
        while has_more {
            let body: serde_json::Value = reqwest::Client::new()
                .get(format!(
                    "https://api.bilibili.com/x/v3/fav/resource/list?media_id={}&pn={}&ps=20",
                    fid, pn
                ))
                .header("Cookie", gen_cookie(sessdata))
                .send()
                .await?
                .json()
                .await?;
            let msg = body["message"].as_str().unwrap_or_default();
            let code = body["code"].as_i64().unwrap_or_default();
            if code != 0 {
                return Err(MyError::BiliError(BError {
                    code,
                    msg: String::from(msg),
                }));
            }
            if let Some(l) = body["data"]["medias"].as_array() {
                for x in l {
                    list.push(x.clone());
                }
            }
            has_more = body["data"]["has_more"].as_bool().unwrap_or_default();
            pn += 1;
        }
        Ok(list)
    }

    pub async fn get_ps(bvid: &str, sessdata: &str) -> Result<serde_json::Value, MyError> {
        let body: serde_json::Value = reqwest::Client::new()
            .get(format!(
                "https://api.bilibili.com/x/player/pagelist?bvid={}",
                bvid
            ))
            .header("Cookie", gen_cookie(sessdata))
            .send()
            .await?
            .json()
            .await?;
        let msg = body["message"].as_str().unwrap_or_default();
        let code = body["code"].as_i64().unwrap_or_default();
        if code != 0 {
            return Err(MyError::BiliError(BError {
                code,
                msg: String::from(msg),
            }));
        }
        Ok(body)
    }

    pub async fn get_url(
        bvid: &str,
        cid: &str,
        sessdata: &str,
    ) -> Result<serde_json::Value, MyError> {
        let body: serde_json::Value = reqwest::Client::new()
            .get(format!(
                "https://api.bilibili.com/x/player/playurl?bvid={}&cid={}&fnval=16",
                bvid, cid
            ))
            .header("Cookie", gen_cookie(sessdata))
            .send()
            .await?
            .json()
            .await?;
        let msg = body["message"].as_str().unwrap_or_default();
        let code = body["code"].as_i64().unwrap_or_default();
        if code != 0 {
            return Err(MyError::BiliError(BError {
                code,
                msg: String::from(msg),
            }));
        }
        Ok(body)
    }

    pub async fn download_music(path: &str, url: &str, sessdata: &str) -> Result<i64, MyError> {
        let body = reqwest::Client::new()
            .get(url)
            .header("Cookie", gen_cookie(sessdata))
            .header("Referer", "https://www.bilibili.com")
            .header("Origin", "https://www.bilibili.com")
            .header(
                "User-Agent",
                " Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:90.0) Gecko/20100101 Firefox/90.0",
            )
            .send()
            .await?
            .bytes()
            .await?;
        let file = std::fs::File::create(path);
        return match file {
            Ok(mut file) => match file.write_all(&body.slice(..)) {
                Ok(_) => Ok(0),
                Err(e) => Err(MyError::BiliError(BError {
                    code: -2,
                    msg: String::from(format!("文件写入失败：{}", e)),
                })),
            },
            Err(e) => Err(MyError::BiliError(BError {
                code: -1,
                msg: String::from(format!("文件创建失败：{}", e)),
            })),
        };
    }
}
