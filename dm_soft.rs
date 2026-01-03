//! Rust 调用 Python 调用大漠插件, 然后将大漠插件的执行结果返回给 Rust.

use crate::rust_tools::json::{Convert, Extract};
use anyhow::{Result, bail};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::thread;

/// find_pic 函数的额外选项.
#[derive(Debug)]
pub struct FindPicOptions {
    pub delta_color: String,
    pub sim: f64,
    pub dir: u8,
}
impl Default for FindPicOptions {
    fn default() -> Self {
        Self {
            delta_color: "000000".to_string(),
            sim: 0.9,
            dir: 0,
        }
    }
}

/// 矩形
#[derive(Debug)]
pub struct Rect {
    pub x1: i64,
    pub y1: i64,
    pub x2: i64,
    pub y2: i64,
}

/// 坐标
#[derive(Debug, Clone)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}

/// 大漠插件.
pub struct DmSoft {
    child: Child,
    id_index: u32,
}
#[allow(dead_code)]
impl DmSoft {
    /// SetPath
    pub fn set_path(&mut self, path: &str) -> Result<bool> {
        let payload = json!({"path": path});
        let v = self.send_task("SetPath", &payload);
        log::debug!("set_path:{payload:?},回复:{v:?}");

        let v = v?;
        let status = Extract::get_str(&v, "status")?;
        if status == "error" {
            let msg = Extract::get_str(&v, "msg")?;
            bail!(msg.to_string())
        }
        Ok(if Extract::get_u64(&v, "result")? == 1 {
            true
        } else {
            false
        })
    }

    /// Reg
    pub fn reg(&mut self, code: &str, ver: &str) -> Result<u64> {
        let payload = json!({"code": code, "ver": ver});
        let v = self.send_task("Reg", &payload);
        log::debug!("reg:{payload:?},回复:{v:?}");

        let v = v?;
        let status = Extract::get_str(&v, "status")?;
        if status == "error" {
            let msg = Extract::get_str(&v, "msg")?;
            bail!(msg.to_string())
        }
        Ok(Extract::get_u64(&v, "result")?)
    }

    /// MoveTo
    pub fn move_to(&mut self, p: &Point) -> Result<bool> {
        let payload = json!({"x": p.x, "y": p.y});
        let v = self.send_task("MoveTo", &payload);
        log::debug!("move_to:{payload:?},回复:{v:?}");

        let v = v?;
        let status = Extract::get_str(&v, "status")?;
        if status == "error" {
            let msg = Extract::get_str(&v, "msg")?;
            bail!(msg.to_string())
        }
        Ok(if Extract::get_u64(&v, "result")? == 1 {
            true
        } else {
            false
        })
    }

    /// LeftClick
    pub fn left_click(&mut self) -> Result<bool> {
        let payload = json!({});
        let v = self.send_task("LeftClick", &payload);
        log::debug!("left_click:{payload:?},回复:{v:?}");

        let v = v?;
        let status = Extract::get_str(&v, "status")?;
        if status == "error" {
            let msg = Extract::get_str(&v, "msg")?;
            bail!(msg.to_string())
        }
        Ok(if Extract::get_u64(&v, "result")? == 1 {
            true
        } else {
            false
        })
    }

    /// FindPic.
    pub fn find_pic(
        &mut self,
        r: &Rect,
        pic_name: &str,
        opts: &FindPicOptions,
    ) -> Result<Option<Point>> {
        let payload = json!({"x1": r.x1, "y1": r.y1, "x2": r.x2, "y2": r.y2,
            "pic_name": pic_name,"delta_color": opts.delta_color, "sim": opts.sim, "dir": opts.dir});
        let v = self.send_task("FindPic", &payload);
        log::debug!("find_pic:{payload:?},回复:{v:?}");

        let v = v?;
        let status = Extract::get_str(&v, "status")?;
        if status == "error" {
            let msg = Extract::get_str(&v, "msg")?;
            bail!(msg.to_string())
        }

        let result = Extract::get_array(&v, "result")?;
        let x = Convert::try_into_i64(&result[0])?;
        let y = Convert::try_into_i64(&result[1])?;

        let r1 = Convert::try_into_i64(&result[2])?;
        if r1 == -1 {
            Ok(None)
        } else {
            Ok(Some(Point { x, y }))
        }
    }

    /// GetWindowRect
    pub fn get_window_rect(&mut self, hwnd: u64) -> Result<Rect> {
        let payload = json!({"hwnd": hwnd});
        let v = self.send_task("GetWindowRect", &payload);
        log::debug!("get_window_rect:{payload:?},回复:{v:?}");

        let v = v?;
        let status = Extract::get_str(&v, "status")?;
        if status == "error" {
            let msg = Extract::get_str(&v, "msg")?;
            bail!(msg.to_string())
        }

        let result = Extract::get_array(&v, "result")?;
        let x1 = Convert::try_into_i64(&result[0])?;
        let y1 = Convert::try_into_i64(&result[1])?;
        let x2 = Convert::try_into_i64(&result[2])?;
        let y2 = Convert::try_into_i64(&result[3])?;

        let e5 = Convert::try_into_i64(&result[4])?;
        if e5 == 0 {
            bail!("获取窗口位置失败")
        }
        Ok(Rect { x1, y1, x2, y2 })
    }
}
#[allow(dead_code)]
impl DmSoft {
    /// 发送任务
    fn send_task(&mut self, cmd: &str, payload: &Value) -> Result<Value> {
        self.id_index += 1;
        if self.id_index == 0 {
            self.id_index = 1;
        }
        let req = json!({"id": self.id_index, "cmd": cmd, "payload": payload});

        let stdin = self.child.stdin.as_mut().unwrap();
        let line = req.to_string() + "\n";
        stdin.write_all(line.as_bytes())?;
        stdin.flush()?;

        let stdout = self.child.stdout.take().unwrap();
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        if reader.read_line(&mut line)? > 0 {
            self.child.stdout = Some(reader.into_inner());
            return Ok(serde_json::from_str(&line)?);
        }

        bail!("获取 Python 回复结果错误.")
    }

    /// 等待 `{"status":"ready"}` 回复.
    pub fn wait_ready(&mut self) -> Result<()> {
        let stdout = self.child.stdout.take().unwrap();
        let mut reader = BufReader::new(stdout);

        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            bail!("等待 READY 回复前 Python 结束.")
        }

        let v: Value = serde_json::from_str(&line)?;
        if v["status"] == "ready" {
            self.child.stdout = Some(reader.into_inner());
            Ok(())
        } else {
            bail!("必须要先回复 READY.")
        }
    }

    /// 创建 DmSoft.
    pub fn new() -> Result<Self> {
        let mut child = Command::new("dm_worker.exe")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // 获取 stderr 防止阻塞.
        let stderr = child.stderr.take().unwrap();
        thread::Builder::new()
            .name("DmSoft-stderr".to_string())
            .spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        log::error!("[PY-ERR] {line}");
                    }
                }
            })?;

        Ok(DmSoft { child, id_index: 0 })
    }
}
