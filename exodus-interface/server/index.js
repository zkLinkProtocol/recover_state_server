const Koa = require('koa')
const koaStatic = require('koa-static')
const koaMount = require('koa-mount')
const path = require('path')
const proxy = require('koa-proxies')
const Router = require('koa-router')
const { getBlocksRowCount } = require('./blocks')

const app = new Koa()
const router = new Router();
const port = process.env.PORT || 80

app.use(
  proxy('/api', {
    target: 'http://127.0.0.1:8080',
    changeOrigin: true,
    rewrite: (path) => path.replace(/\/api/, ''),
  })
)

app.use(koaMount('/', koaStatic(path.resolve(__dirname, '../build'))))

router.get('/server/blocks', async (ctx, next) => {
  try {
    const blocks = await getBlocksRowCount().catch(e => {
      throw new Error(e?.message)
    })
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

app.use(router.routes())

app.listen(port, () => {
  console.log(` Your application is running here: http://localhost:${port}`)
})
