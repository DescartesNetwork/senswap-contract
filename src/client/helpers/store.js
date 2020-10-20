const path = require('path');
const fs = require('fs');

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

  save(uri, config) {
    try {
      fs.mkdirSync(this.dir);
    } catch (er) {
      console.warn(er.message);
    }
    const filename = path.join(this.dir, uri);
    fs.writeFileSync(filename, JSON.stringify(config), 'utf8');
  }
}

module.exports = { Store }
