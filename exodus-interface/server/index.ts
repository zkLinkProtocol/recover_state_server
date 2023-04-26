import Koa from 'koa'
import koaStatic from 'koa-static'
import koaMount from 'koa-mount'
import path from 'path'
import proxy from 'koa-proxies'

const app = new Koa()
const port = process.env.PORT || 80

app.use(
  proxy('/api', {
    target: 'http://127.0.0.1:8080',
    changeOrigin: true,
    rewrite: (path) => path.replace(/^\/api(\/|\/\w+)?$/, '/'),
    logs: true,
  })
)

app.use(koaMount('/', koaStatic(path.resolve(__dirname, '../build'))))

app.listen(port, () => {
  console.log(` Your application is running here: http://localhost:${port}`)
})
