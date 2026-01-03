import ctypes
import os
from typing import Optional, Any

from comtypes import COMError
from comtypes.client import CreateObject


# from win32com.client import CreateObject


class DMInstanceManager:
    _current_dir = os.path.dirname(os.path.abspath(__file__))
    _instance: Optional["DMInstanceManager"] = None
    _instance_store = {}

    def __init__(self):
        dm_reg_path = os.path.join(self._current_dir, "DmReg.dll")
        dm_dll_path = os.path.join(self._current_dir, "dm.dll")

        if not os.path.exists(dm_reg_path):
            raise FileNotFoundError(f"找不到 DmReg.dll：{dm_reg_path}")
        if not os.path.exists(dm_dll_path):
            raise FileNotFoundError(f"找不到 dm.dll：{dm_dll_path}")

        try:
            dms = ctypes.windll.LoadLibrary(dm_reg_path)
            dms.SetDllPathW(dm_dll_path, 0)
        except OSError as e:
            raise RuntimeError(f"加载 DLL 失败：{e}") from e

    def __new__(cls):
        if cls._instance is None:
            cls._instance = super().__new__(cls)
        return cls._instance

    def create_instance(self) -> int:
        """创建 dm 实例并返回实例 ID"""
        try:
            dm = CreateObject('dm.dmsoft')
        except COMError as e:
            raise RuntimeError(f"创建 dm 实例失败：{e}") from e

        instance_id = dm.GetID()
        self._instance_store[instance_id] = dm
        return instance_id

    def get_instance(self, instance_id: int) -> Any:
        """通过实例 ID 获取 dm 实例"""
        dm = self._instance_store.get(instance_id)
        if dm is None:
            raise ValueError(f"未找到 ID 为 {instance_id} 的 dm 实例")
        return dm

    def close_instance(self, instance_id: int) -> bool:
        """通过实例 ID 释放 dm 实例"""
        if instance_id in self._instance_store:
            dm = self._instance_store.pop(instance_id)
            r = dm.ReleaseRef()
            if r == 1:
                return True
            else:
                return False
        else:
            raise ValueError(f"未找到 ID 为 {instance_id} 的 dm 实例")
