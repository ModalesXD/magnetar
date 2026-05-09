(() => {
  const pending = new Map();
  const listeners = new Map();
  let messageId = 0;

  window.__magnetar = (msg) => {
    if (msg.id) {
      const resolve = pending.get(msg.id);
      if (resolve) {
        pending.delete(msg.id);
        resolve(msg.data);
      }
      return;
    }

    if (msg.event) {
      const callbacks = listeners.get(msg.event) || [];
      callbacks.forEach((cb) => cb(msg.data));
    }
  };

  window.magnetar = {
    invoke(command, data = null) {
      return new Promise((resolve) => {
        const id = String(++messageId);
        pending.set(id, resolve);

        try {
          window.webkit.messageHandlers.magnetar.postMessage(
            JSON.stringify({ id, command, data }),
          );
        } catch (e) {
          pending.delete(id);
          resolve(null);
        }
      });
    },

    on(event, callback) {
      if (!listeners.has(event)) {
        listeners.set(event, []);
      }
      listeners.get(event).push(callback);
    },

    once(event, callback) {
      const wrapper = (data) => {
        callback(data);
        const handlers = listeners.get(event);
        if (!handlers) return;
        const index = handlers.indexOf(wrapper);
        if (index !== -1) handlers.splice(index, 1);
      };
      this.on(event, wrapper);
    },
  };
})();
