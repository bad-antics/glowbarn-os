"""
Vibration Sensor for GlowBarn OS.
"""

import random
from typing import Any, Dict
from .base import GPIOSensor, SensorReading


class VibrationSensor(GPIOSensor):
    """
    Vibration/seismic sensor for detecting physical disturbances.
    """
    
    @property
    def sensor_type(self) -> str:
        return "vibration"
    
    @property
    def unit(self) -> str:
        return "g"  # Acceleration in g-force
    
    async def read(self) -> SensorReading:
        """Read vibration level."""
        try:
            # Baseline ambient vibration
            value = abs(random.gauss(0.01, 0.005))
            
            # Occasional spike
            if random.random() < 0.005:
                value += random.uniform(0.1, 0.5)
            
            value = round(value, 4)
            
            metadata = {
                'alert': value > 0.1
            }
            
            return self._create_reading(value, 1.0, metadata)
            
        except Exception as e:
            self.logger.error(f"Vibration read error: {e}")
            return self._create_reading(0.0, 0.0, {'error': str(e)})
