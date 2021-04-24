// TODO fix mime type when serving it
// TODO remove console.logs

function sliderchange(event) {
  console.log("sliderchange: ", event);
  var el = event.srcElement;
  var form = el.closest('form');

  var form_data = new FormData(form);
  console.log("form_data: ", form_data);

  var req_vals = [];
  var entries = form_data.entries();
  for (var pair of form_data.entries()) {
    req_vals.push(encodeURIComponent(pair[0]) + "=" + encodeURIComponent(pair[1]));
  }
  var req_body = req_vals.join("&");

  fetch(form.action, {
    method: form.method,
    body: req_body,
  });
}

window.onload = function onload() {
  var sliders = document.getElementsByClassName("js-chan-slider");

  console.log(sliders);

  for (let i = 0; i < sliders.length; i++) {
    var slider = sliders[i];
    console.log("slider: ", slider);

    slider.oninput = sliderchange;
  };
}
