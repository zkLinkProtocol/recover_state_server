module.exports = {
  apps : [{
    name: 'dunkirk-web-server',
    script: './server/index.js',
    watch: '.',
    env: {
            "PORT": process.env.PORT || 80,
        }
  }]
};
