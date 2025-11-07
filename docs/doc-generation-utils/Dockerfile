FROM pandoc/extra:latest-alpine
ENV TERM=xterm

RUN apk add --no-cache \
    bash \
    curl \
    git \
    inkscape \
    make \
    ncurses \
    perl \
    python3 \
    python3-dev \
    py3-pip \
    py3-setuptools \
    py3-wheel \
    rsync \
    unzip \
    wget \
    zip

RUN tlmgr update --self \
  && tlmgr install \
      catchfile \
      context \
      datetime2 \
      dejavu \
      emoji \
      eso-pic \
      koma-script \
      lastpage \
      lineno \
      minted \
      moresize \
      newunicodechar \
      noto-emoji \
      pgf \
      pgfopts \
      plex \
      reledmac \
      sectsty \
      sourcecodepro \
      svg \
      titlesec \
      tocloft \
      ucs \
      xargs \
      xcolor-material \
      xstring \
  && luaotfload-tool --update \
  && chmod o+w /opt/texlive/texdir/texmf-var 

RUN apk add font-noto font-noto-emoji font-twemoji

# ENTRYPOINT ["sh"]
