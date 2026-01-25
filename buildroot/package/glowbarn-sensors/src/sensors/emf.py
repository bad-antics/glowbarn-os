"""
EMF (Electromagnetic Field) Sensor for GlowBarn OS.
Supports various EMF detector modules and USB meters.
"""

import random
from typing import Any, Dict
from .base import SensorBase, SensorReading, GPIOSensor


class EMFSensor(GPIOSensor):
    """
    EMF Sensor for detecting electromagnetic field fluctuations.
    
    Supports:
    - Analog EMF detectors via ADC
    - K-II style meters via GPIO
    - USB EMF meters via serial
    """
    
    @property
    def sensor_type(self) -> str:
        return "emf"
    
    @property
    def unit(self) -> str:
        return "mG"  # milliGauss
    
    async def initialize(self) -> bool:
        """Initialize EMF sensor."""
        result = await super().initialize()
        if result:
            self.logger.info("EMF sensor ready")
            self._baseline = await self._calibrate()
        return result
    
    async def _calibrate(self) -> float:
        """Calibrate baseline EMF reading."""
        # In production, would take multiple readings to establish baseline
        return 0.3  # Typical ambient EMF
    
    async def read(self) -> SensorReading:
        """
        Read current EMF level.
        
        Returns value in milliGauss (mG).
        Normal ambient: 0.1 - 0.5 mG
        Elevated: 0.5 - 2.0 mG
        High: 2.0 - 5.0 mG
        Very High: > 5.0 mG
        """
        try:
            # In production, would read actual sensor value
            # Simulating realistic EMF readings with occasional spikes
            base_reading = self._baseline + random.gauss(0, 0.1)
            
            # Occasional spike simulation (for demo)
            if random.random() < 0.02:  # 2% chance of spike
                base_reading += random.uniform(1.0, 5.0)
            
            value = max(0.0, round(base_reading, 2))
            
            # Determine quality based on reading stability
            quality = 1.0 if value < 10.0 else 0.8
            
            metadata = {
                'baseline': self._baseline,
                'deviation': value - self._baseline,
                'alert': value > 2.0
            }
            
            return self._create_reading(value, quality, metadata)
            
        except Exception as e:
            self.logger.error(f"EMF read error: {e}")
            return self._create_reading(0.0, 0.0, {'error': str(e)})
    
    def get_alert_level(self, value: float) -> str:
        """Determine alert level based on EMF reading."""
        if value < 0.5:
            return "normal"
        elif value < 2.0:
            return "elevated"
        elif value < 5.0:
            return "high"
        else:
            return "very_high"
