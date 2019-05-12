package com.github.apohrebniak.hoverui.ui;

import com.vaadin.flow.component.button.Button;
import com.vaadin.flow.component.html.Label;
import com.vaadin.flow.component.icon.Icon;
import com.vaadin.flow.component.icon.VaadinIcon;
import com.vaadin.flow.component.orderedlayout.VerticalLayout;
import com.vaadin.flow.component.textfield.TextField;
import com.vaadin.flow.data.binder.Binder;
import com.vaadin.flow.data.value.ValueChangeMode;
import com.vaadin.flow.router.Route;
import lombok.Builder;
import lombok.Data;

@Route("")
public class LoginView extends VerticalLayout {
  private Label title;
  private TextField host;
  private TextField port;
  private Button goButton;

  public LoginView() {
    this.title = new Label("Connect to KV node");
    this.host = new TextField();
    this.port = new TextField();
    this.goButton = new Button();
    setupUI();
  }

  private void setupUI() {
    this.setAlignItems(Alignment.CENTER);

    title.getStyle().set("font-size", "20px");
    title.getStyle().set("font-weight", "bold");
    title.getStyle().set("font-family", "monospace");


    Binder<Input> binder = new Binder<>();
    Input input = Input.builder().build();

    host.setLabel("KV node host");
    host.setPlaceholder("127.0.0.1");
    host.setValueChangeMode(ValueChangeMode.EAGER);
    Binder.Binding<Input, String> hostBinding = binder.forField(host)
        .withValidator(value -> !value.trim().isBlank(), "Cannot be empty")
        .bind(Input::getHost, Input::setHost);
    host.addValueChangeListener(event -> hostBinding.validate());

    port.setLabel("KV node port");
    port.setPlaceholder("5050");
    port.setValueChangeMode(ValueChangeMode.EAGER);
    Binder.Binding<Input, String> portBinding = binder.forField(port)
        .withValidator(value -> !value.trim().isBlank() && value.matches("^[0-9]+$"), "Cannot be empty")
        .bind(Input::getPort, Input::setPort);
    port.addValueChangeListener(event -> portBinding.validate());

    goButton.setText("Connect");
    goButton.setIcon(new Icon(VaadinIcon.GLASSES));
    goButton.addClickListener(event -> {
      if (binder.writeBeanIfValid(input)) {

        goButton.getUI().ifPresent(ui -> ui.navigate("node/" + input.getHost() + "_" + input.getPort()));

        input.setHost(null);
        input.setPort(null);
      }
    });

    add(title);
    add(host);
    add(port);
    add(goButton);
  }

  @Data
  @Builder
  static class Input {
    private String host;
    private String port;
  }
}
