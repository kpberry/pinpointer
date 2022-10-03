import requests
from pathlib import Path
import json

filenames = ['ne_10m_admin_0_countries_lakes', 'ne_10m_admin_1_states_provinces']
for filename in filenames:
    data = requests.get(f'https://raw.githubusercontent.com/nvkelso/natural-earth-vector/master/geojson/{filename}.geojson')
    Path(f'data/{filename}.json').write_text(json.dumps(json.loads(data.text), indent=2), 'utf-8')
