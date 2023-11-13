(function() {
  htmx.config.scrollBehavior = 'auto'

  document.body.addEventListener('htmx:beforeRequest', _ => {
    document.querySelector('#content').scrollTo(0, 10);
  });
}());
