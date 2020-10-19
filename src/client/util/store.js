const path = require('path');
const fs = require('fs');
const mkdirp = require('mkdirp-promise');

class Store {
  dir = path.join(__dirname, '../../../store');

  load(uri) {
    try {
      const filename = path.join(this.dir, uri);
      const data = fs.readFileSync(filename, 'utf8');
      const config = JSON.parse(data);
      return config;
    } catch (er) {
      return null;
    }
  }

  async save(uri, config) {
    await mkdirp(this.dir);
    const filename = path.join(this.dir, uri);
    fs.writeFileSync(filename, JSON.stringify(config), 'utf8');
  }
}

module.exports = { Store };
