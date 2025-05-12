(function() {
  htmx.config.scrollBehavior = 'auto'

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

  // Update URL when scrolling to new messages
  const observer = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        const messageId = entry.target.dataset.messageId;
        if (messageId) {
          const newUrl = `/message/${messageId}`;
          window.history.replaceState({}, '', newUrl);
        }
      }
    });
  }, {
    threshold: 0.5
  });

  // Observe all message containers
  document.addEventListener('htmx:afterSwap', () => {
    document.querySelectorAll('.messages-container').forEach(container => {
      observer.observe(container);
    });
  });
}());
