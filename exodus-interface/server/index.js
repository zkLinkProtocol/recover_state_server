const Koa = require('koa')
const koaStatic = require('koa-static')
const koaMount = require('koa-mount')
const path = require('path')
const proxy = require('koa-proxies')

const app = new Koa()
const port = process.env.PORT || 8081

app.use(
  proxy('/api', {
    target: 'http://127.0.0.1:8080',
    changeOrigin: true,
    rewrite: (path) => path.replace(/\/api/, ''),
    logs: (ctx, target) => {
      console.log(
        '%s - %s %s proxy to -> %s',
        new Date().toISOString(),
        ctx.req.method,
        ctx.req.oldPath,
        new URL(ctx.req.url, target)
      )
    },
  })
)

app.use(koaMount('/', koaStatic(path.resolve(__dirname, '../build'))))

app.listen(port, () => {
  console.log(` Your application is running here: http://localhost:${port}`)
})
