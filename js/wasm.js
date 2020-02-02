const wasm = require('../src/wasm.rs');

wasm.initialize().then(module => {
  console.log('Calling ccall');
  module.ccall('compile', null, ['string', 'string'], ['test', 'hello {{world}}']);
  const add = module.cwrap('add', 'number', ['number', 'number'])
  const compile = module.cwrap('compile', null, ['string', 'string'])

  console.log('Calling rust functions from javascript!')
  console.log(add(1, 5))
  compile('test', 'hello {{world}}');
})
