window.storePlugins = {};
window.countPlugins = () => Object.keys(window.storePlugins).length;
window.loadPlugin   = (name) => window.storePlugins[name];
window.registerPlugin = (name, object) => window.storePlugins[name] = object;