FROM centos:7
EXPOSE 8080
CMD ["/guitar-exercises"]
COPY ./ /