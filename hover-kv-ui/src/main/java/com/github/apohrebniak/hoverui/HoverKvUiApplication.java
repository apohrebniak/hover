package com.github.apohrebniak.hoverui;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.boot.web.client.RestTemplateBuilder;
import org.springframework.context.annotation.Bean;
import org.springframework.web.client.RestTemplate;

import java.time.Duration;

@SpringBootApplication
public class HoverKvUiApplication {

  public static void main(String[] args) {
    SpringApplication.run(HoverKvUiApplication.class, args);
  }

  @Bean
  public RestTemplate restTemplate() {
    return new RestTemplateBuilder()
        .setReadTimeout(Duration.ofSeconds(5))
        .build();
  }
}
