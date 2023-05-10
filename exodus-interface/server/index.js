const Koa = require('koa')
const koaStatic = require('koa-static')
const koaMount = require('koa-mount')
const path = require('path')
const proxy = require('koa-proxies')
const Router = require('koa-router')
const { initRecoverBlocks, getRecoverBlocks } = require('./blocks')
const { initContracts, getContracts } = require('./contracts')

const app = new Koa()
const router = new Router();
const port = process.env.PORT || 80

async function main() {
  app.use(
    proxy('/api', {
      target: 'http://127.0.0.1:8080',
      changeOrigin: true,
      rewrite: (path) => path.replace(/\/api/, ''),
    })
  )

  app.use(koaMount('/', koaStatic(path.resolve(__dirname, '../build'))))

  await initContracts()
  await initRecoverBlocks()

  router.get('/server/blocks', async (ctx, next) => {
    try {
      const blocks = getRecoverBlocks()
      if (blocks === undefined) {
        throw new Error('Blocks synchronization failed')
      }
      ctx.body = {
        code: 0,
        data: {
          blocks
        }
      }
    }
    catch (e) {
      ctx.body = {
        code: 100,
        err_msg: e?.message
      }
    }
  })

  router.get('/server/contracts', async (ctx, next) => {
    try {
      const contracts = await getContracts()

      ctx.body = {
        code: 0,
        data: contracts
      }
    }
    catch (e) {
      ctx.body = {
        code: 100,
        err_msg: e?.message
      }
    }
  })


  app.use(router.routes())

  app.listen(port, () => {
    console.log(` Your application is running here: http://localhost:${port}`)
  })

}

main()