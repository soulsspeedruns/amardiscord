(function() {
  htmx.config.scrollBehavior = 'auto'
  htmx.logAll()

  document.body.addEventListener('htmx:beforeRequest', _ => {
    document.querySelector('#content').scrollTo(0, 10);
  });

  // Function to copy message link to clipboard
  window.copyMessageLink = function(messageId) {
    const url = `${window.location.origin}/message/${messageId}`;
    navigator.clipboard.writeText(url).then(() => {
      // Visual feedback could be added here
    });
  };

  document.body.addEventListener('htmx:beforeSwap', function(evt) {
    if (evt.detail.target.id === 'content') {
      // Preserve scroll position when navigating
      const scrollPos = window.scrollY;
      evt.detail.xhr.addEventListener('load', function() {
        window.scrollTo(0, scrollPos);
      });
    }
  });

  document.body.addEventListener('htmx:afterSwap', function(evt) {
    // Update channel list when loading a channel
    if (evt.detail.target.id === 'content') {
      const match = evt.detail.xhr.responseURL.match(/\/channel\/(\d+)\/\d+/);
      if (match) {
        const channelId = match[1];
        document.getElementById('channels').setAttribute('hx-get', `/channels?current_channel_id=${channelId}`);
        htmx.trigger('#channels', 'load');
      }
    }
  });

  document.body.addEventListener('htmx:afterSettle', function(evt) {
    // Scroll to target message if it exists
    const targetMessage = document.getElementById('target-message');
    if (targetMessage && !targetMessage.hasAttribute('data-scrolled')) {
      targetMessage.scrollIntoView({ behavior: 'smooth', block: 'start' });
      // Remove the ID after scrolling so it only happens once
      targetMessage.setAttribute('data-scrolled', 'true');
    }
  })
}());
