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

  fs.rmdirSync(path.join(tsProject.options.outDir, '/__tests__'), {
    recursive: true,
  });

  done();
});

gulp.task('compile_main', (done) => {
  const basePath = path.join(process.cwd(), 'mod_src');
  const paths = fs.readdirSync(path.join(basePath));

  const executes = [];
  for (const p of paths) {
    if (p[0] == '_') {
      continue;
    }

    const fileList = fs.readdirSync(path.join(basePath, p));
    if (fileList.findIndex((value) => value === 'package.json') === -1) {
      logger.debug(`Root ${p} is invalid environment, skipping...`);
      continue;
    }

    executes.push(
      new Promise((res, rej) =>
        cmd.exec(
          `${process.platform === 'win32' && `powershell`}; cd ${path.join(
            process.cwd(),
            'mod_src',
            p,
          )}; yarn`,
          (e, stdout, stderr) => {
            const _logger = logger.customName(p);
            if (
              stderr.search('Finished') !== -1 ||
              stdout.search('success') !== -1
            ) {
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
      done();
    });
});

gulp.task('compile_move', (done) => {
  const basePath = path.join(process.cwd(), 'mod_src');
  const paths = fs.readdirSync(path.join(basePath));

  if (!fs.existsSync(path.join(process.cwd(), 'src', 'modules'))) {
    fs.mkdirSync(path.join(process.cwd(), 'src', 'modules'));
  }

  for (const p of paths) {
    if (p[0] === '_') {
      continue;
    }
    fs.renameSync(
      path.join(basePath, p, 'pkg'),
      path.join(process.cwd(), 'src', 'modules', p),
    );
  }

  done();
});

gulp.task('compile', gulp.series(['compile_main']));

gulp.task(
  'build',
  gulp.series(['build_pre', 'compile', 'build_main', 'build_post']),
);

// TODO : make initserver command
// gulp.task('initserver',)
