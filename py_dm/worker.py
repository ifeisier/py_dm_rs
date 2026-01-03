import sys
import json
import traceback
from DMInstanceManager import DMInstanceManager


def safe_write(obj):
    try:
        sys.stdout.write(json.dumps(obj, ensure_ascii=False) + "\n")
        sys.stdout.flush()
    except OSError:
        pass


# 创建一个默认的大漠实例，只使用这个大漠作为操作对象
dmm = DMInstanceManager()
dm = dmm.get_instance(dmm.create_instance())
dm.EnableRealMouse(1, 30, 20)


def handle_cmd(task_id, name, payload):
    """处理命令并返回结果"""
    result = 0
    if name == "SetPath":
        result = dm.SetPath(payload["path"])
    elif name == "Reg":
        result = dm.Reg(payload["code"], payload["ver"])
    elif name == "MoveTo":
        result = dm.MoveTo(payload["x"], payload["y"])
    elif name == "LeftClick":
        result = dm.LeftClick()
    elif name == "FindPic":
        result = dm.FindPic(payload["x1"], payload["y1"], payload["x2"], payload["y2"],
                            payload["pic_name"], payload["delta_color"],
                            payload["sim"], payload["dir"])
    elif name == "GetWindowRect":
        result = dm.GetWindowRect(payload["hwnd"])
    else:
        safe_write({"status": "error", "id": task_id, "msg": "No command."})
    safe_write({"status": "ok", "id": task_id, "result": result})


# 必须先主动发送 ready, 主进程才会继续处理任务.
safe_write({"status": "ready"})

# 接收传递过来的命令
for line in sys.stdin:
    try:
        # print(f"[DEBUG] raw line = {line!r}", file=sys.stderr, flush=True)
        req = json.loads(line)
        cmd = req.get("cmd")

        if cmd == "exit":
            safe_write({"status": "bye"})
            break
        handle_cmd(req["id"], cmd, req.get("payload"))

    except Exception as e:
        safe_write({
            "status": "error",
            "msg": str(e),
            "trace": traceback.format_exc()
        })
