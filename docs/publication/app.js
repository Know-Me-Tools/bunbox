'use strict';
if(location.protocol==='file:') document.querySelectorAll('[data-html-control]').forEach(a=>{a.removeAttribute('hx-get');a.removeAttribute('hx-target');a.removeAttribute('hx-swap')});
if(isSecureContext&&'serviceWorker' in navigator&&location.protocol!=='file:') window.addEventListener('load',()=>navigator.serviceWorker.register('./sw.js',{scope:'./'}).catch(()=>{}));
