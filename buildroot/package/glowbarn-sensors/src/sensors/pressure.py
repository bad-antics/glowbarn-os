"""
Pressure Sensor for GlowBarn OS.
Atmospheric pressure monitoring via I2C sensors.
"""

import random
from typing import Any, Dict
from .base import I2CSensor, SensorReading


class PressureSensor(I2CSensor):
    """
    Barometric pressure sensor (BMP280/BME280).
    
    Pressure changes can indicate:
    - Weather changes
    - Altitude variations
    - Anomalous atmospheric disturbances
    """
    
    def __init__(self, sensor_id: str, address: int = 0x76, bus: int = 1,
                 config: Dict[str, Any] = None):
        super().__init__(sensor_id, address, bus, config)
        self._last_pressure = 1013.25  # Sea level standard
    
    @property
    def sensor_type(self) -> str:
        return "pressure"
    
    @property
    def unit(self) -> str:
        return "hPa"
    
    async def read(self) -> SensorReading:
        """Read atmospheric pressure."""
        try:
            # Simulate slow pressure changes
            drift = random.gauss(0, 0.1)
            self._last_pressure = max(980, min(1050, self._last_pressure + drift))
            value = round(self._last_pressure, 2)
            
            metadata = {
                'altitude_approx_m': round((1013.25 - value) * 8.3, 0),
                'trend': 'stable'
            }
            
            return self._create_reading(value, 1.0, metadata)
            
        except Exception as e:
            self.logger.error(f"Pressure read error: {e}")
            return self._create_reading(0.0, 0.0, {'error': str(e)})
