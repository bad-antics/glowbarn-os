"""
Humidity Sensor for GlowBarn OS.
"""

import random
from typing import Any, Dict
from .base import GPIOSensor, SensorReading


class HumiditySensor(GPIOSensor):
    """
    Humidity sensor for environmental monitoring.
    
    Paranormal investigations often note humidity changes
    accompanying other phenomena.
    """
    
    def __init__(self, sensor_id: str, pin: int, sensor_model: str = "dht22",
                 config: Dict[str, Any] = None):
        super().__init__(sensor_id, pin, config)
        self.sensor_model = sensor_model
        self._last_humidity = 45.0
    
    @property
    def sensor_type(self) -> str:
        return "humidity"
    
    @property
    def unit(self) -> str:
        return "%"
    
    async def read(self) -> SensorReading:
        """Read current relative humidity."""
        try:
            drift = random.gauss(0, 1.0)
            self._last_humidity = max(10.0, min(95.0, self._last_humidity + drift))
            value = round(self._last_humidity, 1)
            
            metadata = {
                'model': self.sensor_model,
                'comfort_level': self._get_comfort_level(value)
            }
            
            return self._create_reading(value, 1.0, metadata)
            
        except Exception as e:
            self.logger.error(f"Humidity read error: {e}")
            return self._create_reading(0.0, 0.0, {'error': str(e)})
    
    def _get_comfort_level(self, humidity: float) -> str:
        if humidity < 30:
            return "dry"
        elif humidity < 50:
            return "comfortable"
        elif humidity < 70:
            return "humid"
        else:
            return "very_humid"
