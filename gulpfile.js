const fs = require('fs');
const path = require('path');
const gulp = require('gulp');
const babel = require('gulp-babel');
const ts = require('gulp-typescript');
const cmd = require('child_process');
const tsProject = ts.createProject('tsconfig.json');
const logger = require('clear-logger').default;

gulp.task('build_pre', (done) => {
  if (fs.existsSync(tsProject.options.outDir)) {
    fs.rmdirSync(tsProject.options.outDir, { recursive: true });
  }
  done();
});

gulp.task('build_main', () => {
  const tsResult = tsProject
    .src()
    .pipe(babel())
    .pipe(gulp.dest(tsProject.options.outDir));

  return tsResult;
});

gulp.task('build_post', (done) => {
  // FOR Plug_N_Play

  const indexFileRoot = path.join(tsProject.options.outDir, '/index.js');

  const indexFile = fs.readFileSync(indexFileRoot).toString();

  let newIndexFile = '';

  const splitResult = indexFile.split(';', 1);
  splitResult.push(indexFile.substring(splitResult[0].length));

  newIndexFile += splitResult[0];
  // newIndexFile += ";\n\nrequire('../.pnp.js').setup()";
  newIndexFile += splitResult[1];

  fs.writeFileSync(indexFileRoot, newIndexFile);

  fs.rmdir(path.join(tsProject.options.outDir, '/__tests__'), {
    recursive: true,
  });

  done();
});

gulp.task('build', gulp.series(['build_pre', 'build_main', 'build_post']));

gulp.task('compile', (done) => {
  const basePath = path.join(process.cwd(), 'src', 'modules');
  const paths = fs.readdirSync(path.join(basePath));

  const executes = [];
  for (const p of paths) {
    executes.push(
      new Promise((res, rej) =>
        cmd.exec(
          `${process.platform === 'win32' && `powershell`}; cd ./${path.join(
            'src',
            'modules',
            p,
          )}; yarn`,
          (e, stdout, stderr) => {
            console.log(`Dir : ${p}`);
            const _logger = logger.customName(p);
            if (stderr.search('Finished') !== -1) {
              _logger.success(stderr);
              res();
            } else {
              _logger.debug(e, false);
              _logger.debug(stdout, false);
              _logger.debug(`${stderr}`, false);
              rej(stdout);
            }
          },
        ),
      ),
    );
  }

  Promise.all(executes)
    .then(() => {
      done();
    })
    .catch((e) => {
      logger.debug(e);
      process.exit(1);
    });
});
