FROM alpine:3.15

ARG DOCKER_GUID=1000 \
	DOCKER_UID=1000 \
	DOCKER_TIME_CONT=America \
	DOCKER_TIME_CITY=New_York \
	DOCKER_APP_USER=app_user \
	DOCKER_APP_GROUP=app_group

ENV VIRT=".build_packages"
ENV TZ=${DOCKER_TIME_CONT}/${DOCKER_TIME_CITY}

RUN addgroup -g ${DOCKER_GUID} -S ${DOCKER_APP_GROUP} \
	&& adduser -u ${DOCKER_UID} -S -G ${DOCKER_APP_GROUP} ${DOCKER_APP_USER} \
	&& apk --no-cache add --virtual ${VIRT} tzdata \
	&& cp /usr/share/zoneinfo/${TZ} /etc/localtime \
	&& echo ${TZ} > /etc/timezone \
	&& apk del ${VIRT}

WORKDIR /app

RUN mkdir /db_data \
	&& chown ${DOCKER_APP_USER}:${DOCKER_APP_GROUP} /app /db_data

COPY --chown=${DOCKER_APP_USER}:${DOCKER_APP_GROUP} belugasnooze ./

USER ${DOCKER_APP_USER}

CMD [ "./belugasnooze"]