set foldlevel=0

let b:project_root_dir = expand('$HOME/git/todors')

let b:ft_cmds = {
        \ 'rust': {
        \   'build': ' just build',
        \   'build-release': ' just build-release',
        \   'install': ' just install',
        \   'run': ' just run',
        \  },
        \ }
