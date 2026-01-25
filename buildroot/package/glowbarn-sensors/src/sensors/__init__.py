"""
GlowBarn Sensors Library
Provides unified access to all supported paranormal detection sensors.
"""

from .base import SensorBase, SensorReading
from .emf import EMFSensor
from .temperature import TemperatureSensor
from .humidity import HumiditySensor
from .motion import MotionSensor
from .vibration import VibrationSensor
from .pressure import PressureSensor

__all__ = [
    'SensorBase',
    'SensorReading',
    'EMFSensor',
    'TemperatureSensor',
    'HumiditySensor',
    'MotionSensor',
    'VibrationSensor',
    'PressureSensor',
]

__version__ = "1.0.0"
